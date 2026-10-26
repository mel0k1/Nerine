use core::ptr;
use core::ptr::addr_of;

pub const MEM_USABLE: u64 = 0;

#[repr(C)]
pub struct MemEntry {
    pub base: u64,
    pub length: u64,
    pub kind: u64,
}

const COMMON0: u64 = 0xc7b1dd30df4c8b88;
const COMMON1: u64 = 0x0a82e883a194f07b;

const BLOB_LEN: usize = 28;

#[used]
#[link_section = ".limine_requests"]
static mut BLOB: [u64; BLOB_LEN] = {
    let mut b = [0u64; BLOB_LEN];
    b[0] = 0xf6b8f4b39de7d1ae;
    b[1] = 0xfab91a6940fcb9cf;
    b[2] = 0x785c6ed015d3e316;
    b[3] = 0x181e920a7852b9d9;
    b[4] = 0xf9562b2d5c95a6c8;
    b[5] = 0x6a7b384944536bdc;
    b[6] = 1;
    b[7] = COMMON0;
    b[8] = COMMON1;
    b[9] = 0x48dcf1cb8ad2b852;
    b[10] = 0x63984e959a98244b;
    b[11] = 0;
    b[12] = 0;
    b[13] = COMMON0;
    b[14] = COMMON1;
    b[15] = 0x67cf3d9d378a806f;
    b[16] = 0xe304acdfc50c3c62;
    b[17] = 0;
    b[18] = 0;
    b[19] = COMMON0;
    b[20] = COMMON1;
    b[21] = 0x224ef0460a8e8926;
    b[22] = 0xe1cb0fc25f46ea3d;
    b[23] = 0;
    b[24] = 0;
    b[25] = 0x40000;
    b[26] = 0xadc0e0531bb10d03;
    b[27] = 0x9572709f31764c62;
    b
};

fn base() -> *const u64 {
    addr_of!(BLOB) as *const u64
}

fn word(idx: usize) -> u64 {
    unsafe { ptr::read_volatile(base().add(idx)) }
}

pub fn ready() -> bool {
    word(6) == 0 && word(12) != 0 && word(18) != 0
}

pub fn hhdm() -> u64 {
    let r = word(12);
    unsafe { ptr::read_volatile((r as *const u64).add(1)) }
}

pub fn memmap() -> Option<(*const *const MemEntry, u64)> {
    let r = word(18);
    if r == 0 {
        return None;
    }
    unsafe {
        let p = r as *const u64;
        let count = ptr::read_volatile(p.add(1));
        let entries = ptr::read_volatile(p.add(2)) as *const *const MemEntry;
        if count == 0 || entries.is_null() {
            return None;
        }
        Some((entries, count))
    }
}
