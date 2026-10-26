use crate::boot;
use nectar::spin::Spinlock;

const PAGE: u64 = 4096;

#[derive(Clone, Copy)]
struct Range {
    base: u64,
    len: u64,
}

struct Pool {
    ranges: [Range; 64],
    n: usize,
    ri: usize,
    off: u64,
}

impl Pool {
    const EMPTY: Self = Self {
        ranges: [Range { base: 0, len: 0 }; 64],
        n: 0,
        ri: 0,
        off: 0,
    };
}

static POOL: Spinlock<Pool> = Spinlock::new(Pool::EMPTY);

pub fn init() -> bool {
    let Some((entries, count)) = boot::memmap() else {
        return false;
    };
    let mut p = POOL.lock();
    for i in 0..count as usize {
        if p.n >= p.ranges.len() {
            break;
        }
        let e = unsafe { &*entries.add(i) };
        if e.kind != boot::MEM_USABLE {
            continue;
        }
        let mut base = e.base;
        let mut len = e.length;
        if base % PAGE != 0 {
            let d = PAGE - base % PAGE;
            if d > len {
                continue;
            }
            base += d;
            len -= d;
        }
        if len < PAGE {
            continue;
        }
        p.ranges[p.n] = Range { base, len };
        p.n += 1;
    }
    p.n > 0
}

pub fn alloc() -> Option<u64> {
    let mut p = POOL.lock();
    while p.ri < p.n {
        let r = p.ranges[p.ri];
        if p.off + PAGE <= r.len {
            let a = r.base + p.off;
            p.off += PAGE;
            return Some(a);
        }
        p.ri += 1;
        p.off = 0;
    }
    None
}

pub fn zalloc() -> Option<u64> {
    let a = alloc()?;
    unsafe {
        core::ptr::write_bytes(hhdm_virt(a) as *mut u8, 0, PAGE as usize);
    }
    Some(a)
}

pub fn hhdm_virt(phys: u64) -> u64 {
    phys + boot::hhdm()
}
