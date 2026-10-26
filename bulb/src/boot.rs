use core::ptr::addr_of;
use core::sync::atomic::{AtomicU64, Ordering};

const COMMON: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

pub const MEM_USABLE: u64 = 0;

#[repr(C)]
pub struct MemEntry {
    pub base: u64,
    pub length: u64,
    pub kind: u64,
}

#[repr(C, align(8))]
struct Req {
    id: [u64; 4],
    revision: u64,
    response: AtomicU64,
}

#[repr(C, align(8))]
struct StackReq {
    id: [u64; 4],
    revision: u64,
    response: AtomicU64,
    stack_size: u64,
}

#[used]
#[link_section = ".limine_requests"]
static START: [u64; 4] = [0xf6b8f4b39de7d1ae, 0xfab91a6940fcb9cf, 0x785c6ed015d3e316, 0x181e920a7852b9d9];

#[used]
#[link_section = ".limine_requests"]
static BASE_REV: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 1];

#[used]
#[link_section = ".limine_requests"]
static HHDM_REQ: Req = Req {
    id: [COMMON[0], COMMON[1], 0x48dcf1cb8ad2b852, 0x63984e959a98244b],
    revision: 0,
    response: AtomicU64::new(0),
};

#[used]
#[link_section = ".limine_requests"]
static MEMMAP_REQ: Req = Req {
    id: [COMMON[0], COMMON[1], 0x67cf3d9d378a806f, 0xe304acdfc50c3c62],
    revision: 0,
    response: AtomicU64::new(0),
};

#[used]
#[link_section = ".limine_requests"]
static STACK_REQ: StackReq = StackReq {
    id: [COMMON[0], COMMON[1], 0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d],
    revision: 0,
    response: AtomicU64::new(0),
    stack_size: 0x40000,
};

#[used]
#[link_section = ".limine_requests"]
static END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

pub fn ready() -> bool {
    let hhdm = HHDM_REQ.response.load(Ordering::Acquire);
    let memmap = MEMMAP_REQ.response.load(Ordering::Acquire);
    let rev_ok = unsafe { addr_of!(BASE_REV[2]).read_volatile() } == 0;
    hhdm != 0 && memmap != 0 && rev_ok
}

pub fn hhdm() -> u64 {
    let resp = HHDM_REQ.response.load(Ordering::Acquire);
    unsafe { (resp as *const u64).add(1).read_volatile() }
}

pub fn memmap() -> Option<(*const *const MemEntry, u64)> {
    let resp = MEMMAP_REQ.response.load(Ordering::Acquire);
    if resp == 0 {
        return None;
    }
    unsafe {
        let p = resp as *const u64;
        let count = p.add(1).read_volatile();
        let entries = p.add(2).read_volatile() as *const *const MemEntry;
        if count == 0 || entries.is_null() {
            return None;
        }
        Some((entries, count))
    }
}
