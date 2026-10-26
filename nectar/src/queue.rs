use crate::msg::Msg;
use core::mem::{size_of, MaybeUninit};
use core::ptr;
use core::sync::atomic::{fence, AtomicU32, Ordering};

pub const MAGIC: u32 = 0x4E43_5451;
pub const VERSION: u16 = 1;
pub const MSG_SIZE: usize = 32;

#[derive(Debug, PartialEq, Eq)]
pub enum QErr {
    TooSmall,
    Misaligned,
    Magic,
    Version,
    MsgSize,
    Capacity,
}

#[repr(C)]
struct Header {
    magic: u32,
    version: u16,
    msg_size: u16,
    capacity: u32,
    head: AtomicU32,
    tail: AtomicU32,
    doorbell: AtomicU32,
}

pub struct Queue {
    hdr: *mut Header,
    slots: *mut MaybeUninit<Msg>,
    capacity: u32,
}

unsafe impl Sync for Queue {}
unsafe impl Send for Queue {}

impl Queue {
    pub fn init(buf: &mut [u8], capacity: u32) -> Result<Self, QErr> {
        if capacity == 0 {
            return Err(QErr::Capacity);
        }
        if buf.len() < size_of::<Header>() + capacity as usize * MSG_SIZE {
            return Err(QErr::TooSmall);
        }
        if (buf.as_ptr() as usize) % 8 != 0 {
            return Err(QErr::Misaligned);
        }
        let hdr = buf.as_mut_ptr() as *mut Header;
        unsafe {
            ptr::write(
                hdr,
                Header {
                    magic: MAGIC,
                    version: VERSION,
                    msg_size: MSG_SIZE as u16,
                    capacity,
                    head: AtomicU32::new(0),
                    tail: AtomicU32::new(0),
                    doorbell: AtomicU32::new(0),
                },
            );
        }
        Ok(Self {
            hdr,
            slots: unsafe { (hdr as *mut u8).add(size_of::<Header>()) } as *mut MaybeUninit<Msg>,
            capacity,
        })
    }

    pub fn attach(buf: &[u8]) -> Result<Self, QErr> {
        if (buf.as_ptr() as usize) % 8 != 0 {
            return Err(QErr::Misaligned);
        }
        if buf.len() < size_of::<Header>() {
            return Err(QErr::TooSmall);
        }
        let hdr = buf.as_ptr() as *const Header as *mut Header;
        let (magic, version, msg_size, capacity) = unsafe {
            (
                (*hdr).magic,
                (*hdr).version,
                (*hdr).msg_size,
                (*hdr).capacity,
            )
        };
        if magic != MAGIC {
            return Err(QErr::Magic);
        }
        if version != VERSION {
            return Err(QErr::Version);
        }
        if msg_size as usize != MSG_SIZE {
            return Err(QErr::MsgSize);
        }
        if capacity == 0
            || buf.len() < size_of::<Header>() + capacity as usize * MSG_SIZE
        {
            return Err(QErr::TooSmall);
        }
        Ok(Self {
            hdr,
            slots: unsafe { (hdr as *const u8).add(size_of::<Header>()) } as *mut MaybeUninit<Msg>,
            capacity,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn doorbell(&self) -> u32 {
        unsafe { (*self.hdr).doorbell.load(Ordering::Acquire) }
    }

    pub fn try_send(&self, msg: Msg) -> Result<(), Msg> {
        let head = unsafe { (*self.hdr).head.load(Ordering::Acquire) };
        let tail = unsafe { (*self.hdr).tail.load(Ordering::Relaxed) };
        if tail.wrapping_sub(head) >= self.capacity {
            return Err(msg);
        }
        unsafe {
            (*self.slots.add((tail % self.capacity) as usize)).write(msg);
        }
        fence(Ordering::Release);
        unsafe { (*self.hdr).tail.store(tail.wrapping_add(1), Ordering::Release) };
        unsafe { (*self.hdr).doorbell.fetch_add(1, Ordering::AcqRel) };
        Ok(())
    }

    pub fn try_recv(&self) -> Result<Msg, crate::spsc::Empty> {
        let tail = unsafe { (*self.hdr).tail.load(Ordering::Acquire) };
        let head = unsafe { (*self.hdr).head.load(Ordering::Relaxed) };
        if head == tail {
            return Err(crate::spsc::Empty);
        }
        let msg = unsafe { (*self.slots.add((head % self.capacity) as usize)).assume_init_read() };
        fence(Ordering::Release);
        unsafe { (*self.hdr).head.store(head.wrapping_add(1), Ordering::Release) };
        Ok(msg)
    }

    pub fn drain(&self, out: &mut [Msg]) -> usize {
        let mut n = 0;
        while n < out.len() {
            match self.try_recv() {
                Ok(m) => out[n] = m,
                Err(_) => break,
            }
            n += 1;
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(cap: u32) -> Box<[u8]> {
        let len = size_of::<Header>() + cap as usize * MSG_SIZE;
        let mut b = vec![0u8; len].into_boxed_slice();
        let p = b.as_mut_ptr() as usize;
        assert_eq!(p % 8, 0, "allocator must 8-align");
        b
    }

    #[test]
    fn init_attach_roundtrip() {
        let mut mem = buf(4);
        let q = Queue::init(&mut mem, 4).unwrap();
        assert_eq!(q.capacity(), 4);
        let q = Queue::attach(&mem).unwrap();
        for i in 0..4u64 {
            assert!(q.try_send(Msg::new(2, i)).is_ok());
        }
        assert!(q.try_send(Msg::new(2, 99)).is_err());
        let mut out = [Msg::new(0, 0); 8];
        let n = q.drain(&mut out);
        assert_eq!(n, 4);
        for (i, m) in out[..n].iter().enumerate() {
            assert_eq!(m.seq, i as u64);
        }
        assert!(q.try_recv().is_err());
    }

    #[test]
    fn validation() {
        let mut mem = buf(4);
        assert_eq!(Queue::init(&mut mem, 0).err(), Some(QErr::Capacity));
        let mut empty = [0u8; 0];
        assert_eq!(Queue::init(&mut empty, 4).err(), Some(QErr::TooSmall));
        let mut small = [0u8; size_of::<Header>()];
        assert_eq!(Queue::init(&mut small, 4).err(), Some(QErr::TooSmall));
        let q = Queue::init(&mut mem, 4).unwrap();
        let mut corrupt = mem.clone();
        corrupt[0] = 0;
        assert_eq!(Queue::attach(&corrupt).err(), Some(QErr::Magic));
        let _ = q;
    }

    #[test]
    fn threads() {
        let mut mem = buf(32);
        let q = Queue::init(&mut mem, 32).unwrap();
        std::thread::scope(|s| {
            s.spawn(|| {
                for i in 0..5000u64 {
                    while q.try_send(Msg::new(3, i)).is_err() {
                        core::hint::spin_loop();
                    }
                }
            });
            let consumer = s.spawn(|| {
                let mut got = 0usize;
                let mut slot = [Msg::new(0, 0); 16];
                while got < 5000 {
                    let batch = q.drain(&mut slot);
                    if batch == 0 {
                        core::hint::spin_loop();
                    }
                    got += batch;
                }
                got
            });
            assert_eq!(consumer.join().unwrap(), 5000);
        });
        assert!(q.doorbell() >= 5000);
    }
}
