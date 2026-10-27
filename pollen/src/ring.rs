use crate::fastpath::Petal;
use crate::grain::Grain;
use core::mem::size_of;
use core::ptr;
use core::slice;
use core::sync::atomic::{fence, AtomicBool, AtomicU32, Ordering};

pub const MAGIC: u32 = 0x504C_4C4E;
pub const VERSION: u16 = 1;
pub const LEN_PREFIX: u32 = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum RErr {
    TooSmall,
    Misaligned,
    Magic,
    Version,
    Capacity,
    SlotSize,
    TooBig,
}

#[repr(C)]
struct Header {
    magic: u32,
    version: u16,
    _pad: [u8; 2],
    slot_size: u32,
    capacity: u32,
    head: AtomicU32,
    tail: AtomicU32,
    doorbell: AtomicU32,
    petal_out: AtomicU32,
}

pub struct Ring {
    hdr: *mut Header,
    pool: *mut u8,
    stride: u32,
    petal: Option<Petal>,
    granted: AtomicBool,
    claimed: AtomicBool,
}

unsafe impl Sync for Ring {}
unsafe impl Send for Ring {}

impl Ring {
    pub fn init(
        buf: &mut [u8],
        capacity: u32,
        slot_size: u32,
        petal: Option<Petal>,
    ) -> Result<Self, RErr> {
        if capacity == 0 {
            return Err(RErr::Capacity);
        }
        if slot_size == 0 {
            return Err(RErr::SlotSize);
        }
        let stride = LEN_PREFIX as usize + slot_size as usize;
        if stride > u32::MAX as usize {
            return Err(RErr::TooBig);
        }
        let need = size_of::<Header>() + capacity as usize * stride;
        if need > u32::MAX as usize {
            return Err(RErr::TooBig);
        }
        if buf.len() < need {
            return Err(RErr::TooSmall);
        }
        if (buf.as_ptr() as usize) % 4 != 0 {
            return Err(RErr::Misaligned);
        }
        let hdr = buf.as_mut_ptr() as *mut Header;
        unsafe {
            ptr::write(
                hdr,
                Header {
                    magic: MAGIC,
                    version: VERSION,
                    _pad: [0; 2],
                    slot_size,
                    capacity,
                    head: AtomicU32::new(0),
                    tail: AtomicU32::new(0),
                    doorbell: AtomicU32::new(0),
                    petal_out: AtomicU32::new(0),
                },
            );
        }
        Ok(Self {
            hdr,
            pool: unsafe { (hdr as *mut u8).add(size_of::<Header>()) },
            stride: stride as u32,
            petal,
            granted: AtomicBool::new(false),
            claimed: AtomicBool::new(false),
        })
    }

    pub fn attach(buf: &[u8]) -> Result<Self, RErr> {
        if (buf.as_ptr() as usize) % 4 != 0 {
            return Err(RErr::Misaligned);
        }
        if buf.len() < size_of::<Header>() {
            return Err(RErr::TooSmall);
        }
        let hdr = buf.as_ptr() as *const Header as *mut Header;
        let (magic, version, slot_size, capacity) = unsafe {
            (
                (*hdr).magic,
                (*hdr).version,
                (*hdr).slot_size,
                (*hdr).capacity,
            )
        };
        if magic != MAGIC {
            return Err(RErr::Magic);
        }
        if version != VERSION {
            return Err(RErr::Version);
        }
        if capacity == 0 {
            return Err(RErr::Capacity);
        }
        if slot_size == 0 {
            return Err(RErr::SlotSize);
        }
        if slot_size > u32::MAX - LEN_PREFIX {
            return Err(RErr::TooBig);
        }
        let stride = LEN_PREFIX + slot_size;
        if buf.len() < size_of::<Header>() + capacity as usize * stride as usize {
            return Err(RErr::TooSmall);
        }
        Ok(Self {
            hdr,
            pool: unsafe { (hdr as *mut u8).add(size_of::<Header>()) },
            stride,
            petal: None,
            granted: AtomicBool::new(false),
            claimed: AtomicBool::new(false),
        })
    }

    pub fn capacity(&self) -> u32 {
        unsafe { (*self.hdr).capacity }
    }

    pub fn slot_size(&self) -> u32 {
        unsafe { (*self.hdr).slot_size }
    }

    pub fn doorbell(&self) -> u32 {
        unsafe { (*self.hdr).doorbell.load(Ordering::Acquire) }
    }

    pub fn pending(&self) -> u32 {
        let t = unsafe { (*self.hdr).tail.load(Ordering::Acquire) };
        let h = unsafe { (*self.hdr).head.load(Ordering::Acquire) };
        t.wrapping_sub(h)
    }

    pub fn fast_result(&self) -> u32 {
        unsafe { (*self.hdr).petal_out.load(Ordering::Acquire) }
    }

    fn slot(&self, i: u32) -> *mut u8 {
        unsafe { self.pool.add(i as usize * self.stride as usize) }
    }

    fn payload(&self, i: u32) -> *mut u8 {
        unsafe { self.slot(i).add(LEN_PREFIX as usize) }
    }

    pub fn grant(&self) -> Option<&mut [u8]> {
        if self.granted.swap(true, Ordering::AcqRel) {
            return None;
        }
        let h = unsafe { (*self.hdr).head.load(Ordering::Acquire) };
        let t = unsafe { (*self.hdr).tail.load(Ordering::Relaxed) };
        let cap = unsafe { (*self.hdr).capacity };
        if t.wrapping_sub(h) >= cap {
            self.granted.store(false, Ordering::Release);
            return None;
        }
        let ss = unsafe { (*self.hdr).slot_size };
        Some(unsafe { slice::from_raw_parts_mut(self.payload(t % cap), ss as usize) })
    }

    pub fn publish(&self, len: u32) -> bool {
        if !self.granted.swap(false, Ordering::AcqRel) {
            return false;
        }
        let ss = unsafe { (*self.hdr).slot_size };
        if len > ss {
            return false;
        }
        let t = unsafe { (*self.hdr).tail.load(Ordering::Relaxed) };
        let cap = unsafe { (*self.hdr).capacity };
        let i = t % cap;
        unsafe { ptr::write_unaligned(self.slot(i) as *mut u32, len) };
        if let Some(petal) = &self.petal {
            let grain = unsafe { slice::from_raw_parts_mut(self.payload(i), ss as usize) };
            let out = (petal.func)(t.wrapping_add(1), &petal.params, grain);
            unsafe { (*self.hdr).petal_out.store(out, Ordering::Release) };
        }
        fence(Ordering::Release);
        unsafe { (*self.hdr).tail.store(t.wrapping_add(1), Ordering::Release) };
        unsafe { (*self.hdr).doorbell.fetch_add(1, Ordering::AcqRel) };
        true
    }

    pub fn poll(&self) -> Option<Grain> {
        if self.claimed.load(Ordering::Acquire) {
            return None;
        }
        let t = unsafe { (*self.hdr).tail.load(Ordering::Acquire) };
        let h = unsafe { (*self.hdr).head.load(Ordering::Relaxed) };
        if h == t {
            return None;
        }
        let cap = unsafe { (*self.hdr).capacity };
        let ss = unsafe { (*self.hdr).slot_size };
        let i = h % cap;
        let mut len = unsafe { (self.slot(i) as *const u32).read_unaligned() };
        if len > ss {
            len = ss;
        }
        self.claimed.store(true, Ordering::Release);
        let off = LEN_PREFIX as usize + i as usize * self.stride as usize;
        Some(Grain::new(off as u32, len))
    }

    pub fn data(&self, grain: &Grain) -> &[u8] {
        let total = unsafe { (*self.hdr).capacity } as usize * self.stride as usize;
        let off = grain.offset as usize;
        if off >= total {
            return &[];
        }
        let len = (grain.len as usize).min(total - off);
        unsafe { slice::from_raw_parts(self.pool.add(off), len) }
    }

    pub fn release(&self) -> bool {
        if !self.claimed.swap(false, Ordering::AcqRel) {
            return false;
        }
        let h = unsafe { (*self.hdr).head.load(Ordering::Relaxed) };
        fence(Ordering::Release);
        unsafe { (*self.hdr).head.store(h.wrapping_add(1), Ordering::Release) };
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fastpath::Petal;

    fn buf(cap: u32, ss: u32) -> Box<[u8]> {
        let len = size_of::<Header>() + cap as usize * (LEN_PREFIX + ss) as usize;
        vec![0u8; len].into_boxed_slice()
    }

    fn put(ring: &Ring, len: u32) {
        ring.grant().unwrap();
        assert!(ring.publish(len));
    }

    #[test]
    fn roundtrip_in_place() {
        let mut mem = buf(4, 64);
        let ring = Ring::init(&mut mem, 4, 64, None).unwrap();
        let base = ring.pool as usize;
        let mut src = [0u8; 64];
        for (i, b) in src.iter_mut().enumerate() {
            *b = i as u8;
        }
        let grant_ptr = {
            let g = ring.grant().unwrap();
            g.copy_from_slice(&src);
            g.as_ptr() as usize
        };
        assert!(ring.publish(64));
        assert_eq!(ring.pending(), 1);
        let grain = ring.poll().unwrap();
        let data = ring.data(&grain);
        assert_eq!(data, &src[..]);
        assert_eq!(data.as_ptr() as usize, grant_ptr);
        let pool_end = base + 4 * (LEN_PREFIX + 64) as usize;
        assert!(grant_ptr >= base + 4 && grant_ptr + 64 <= pool_end);
        assert!(ring.release());
        assert_eq!(ring.pending(), 0);
        assert!(ring.poll().is_none());
    }

    #[test]
    fn full_and_wrap() {
        let mut mem = buf(2, 16);
        let ring = Ring::init(&mut mem, 2, 16, None).unwrap();
        put(&ring, 1);
        put(&ring, 2);
        assert!(ring.grant().is_none());
        assert_eq!(ring.pending(), 2);
        assert!(!ring.publish(3));
        let g = ring.poll().unwrap();
        assert_eq!(g.len, 1);
        assert!(ring.poll().is_none());
        assert!(ring.release());
        assert!(!ring.release());
        put(&ring, 3);
        assert_eq!(ring.pending(), 2);
        let x = ring.poll().unwrap();
        assert_eq!(x.len, 2);
        assert!(ring.release());
        let y = ring.poll().unwrap();
        assert_eq!(y.len, 3);
        assert!(ring.release());
        assert!(ring.poll().is_none());
        assert_eq!(ring.pending(), 0);
    }

    #[test]
    fn grant_discipline() {
        let mut mem = buf(4, 16);
        let ring = Ring::init(&mut mem, 4, 16, None).unwrap();
        ring.grant().unwrap();
        assert!(ring.grant().is_none());
        assert!(ring.publish(4));
        assert!(ring.grant().is_some());
    }

    #[test]
    fn validation() {
        let mut mem = buf(4, 32);
        assert_eq!(Ring::init(&mut mem, 0, 32, None).err(), Some(RErr::Capacity));
        assert_eq!(Ring::init(&mut mem, 4, 0, None).err(), Some(RErr::SlotSize));
        let mut empty: [u8; 0] = [];
        assert_eq!(Ring::init(&mut empty, 4, 32, None).err(), Some(RErr::TooSmall));
        let mut off = vec![0u8; buf(4, 32).len() + 1].into_boxed_slice();
        assert_eq!(Ring::init(&mut off[1..], 4, 32, None).err(), Some(RErr::Misaligned));
        let _ring = Ring::init(&mut mem, 4, 32, None).unwrap();
        let mut corrupt = mem.clone();
        corrupt[0] = 0;
        assert_eq!(Ring::attach(&corrupt).err(), Some(RErr::Magic));
        let mut badver = mem.clone();
        badver[5] = 1;
        assert_eq!(Ring::attach(&badver).err(), Some(RErr::Version));
        let shared = Ring::attach(&mem).unwrap();
        assert_eq!(shared.capacity(), 4);
        assert_eq!(shared.slot_size(), 32);
        assert_eq!(shared.pending(), 0);
    }

    #[test]
    fn fast_path_runs_at_publish() {
        fn sum(_d: u32, _p: &[u8], g: &mut [u8]) -> u32 {
            g.iter().map(|b| *b as u32).sum()
        }
        let mut mem = buf(4, 64);
        let ring = Ring::init(&mut mem, 4, 64, Some(Petal::new(sum, [0; 16]))).unwrap();
        {
            let g = ring.grant().unwrap();
            g[..4].copy_from_slice(&[1, 2, 3, 4]);
        }
        assert!(ring.publish(4));
        assert_eq!(ring.fast_result(), 10);
        let grain = ring.poll().unwrap();
        assert_eq!(ring.data(&grain)[..4], [1, 2, 3, 4]);
        assert!(ring.release());
    }

    #[test]
    fn fast_path_transforms_payload() {
        fn xor(_d: u32, p: &[u8], g: &mut [u8]) -> u32 {
            for b in g.iter_mut() {
                *b ^= p[0];
            }
            g.len() as u32
        }
        let mut mem = buf(4, 64);
        let ring = Ring::init(&mut mem, 4, 64, Some(Petal::new(xor, [0xFF; 16]))).unwrap();
        {
            let g = ring.grant().unwrap();
            g[..8].fill(0);
        }
        assert!(ring.publish(8));
        let grain = ring.poll().unwrap();
        assert_eq!(ring.data(&grain)[..8], [0xFF; 8]);
        assert_eq!(ring.fast_result(), 64);
        assert!(ring.release());
    }

    #[test]
    fn threads() {
        let mut mem = buf(16, 32);
        let ring = Ring::init(&mut mem, 16, 32, None).unwrap();
        let total = 20_000u32;
        std::thread::scope(|s| {
            s.spawn(|| {
                for i in 0..total {
                    loop {
                        match ring.grant() {
                            Some(g) => {
                                g[..4].copy_from_slice(&i.to_le_bytes());
                                g[4..].fill(i as u8);
                                break;
                            }
                            None => core::hint::spin_loop(),
                        }
                    }
                    assert!(ring.publish(32));
                }
            });
            let consumer = s.spawn(|| {
                let mut got = 0u32;
                while got < total {
                    match ring.poll() {
                        Some(g) => {
                            let d = ring.data(&g);
                            assert_eq!(u32::from_le_bytes(d[..4].try_into().unwrap()), got);
                            for b in d[4..].iter() {
                                assert_eq!(*b, got as u8);
                            }
                            assert!(ring.release());
                            got += 1;
                        }
                        None => core::hint::spin_loop(),
                    }
                }
                got
            });
            assert_eq!(consumer.join().unwrap(), total);
        });
        assert_eq!(ring.doorbell(), total);
        assert_eq!(ring.pending(), 0);
    }
}
