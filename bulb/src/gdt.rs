use core::arch::asm;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

const CODE64: u64 = 0x00AF_9B00_0000_FFFF;
const DATA64: u64 = 0x008F_9300_0000_FFFF;

#[repr(C, align(8))]
struct Gdt {
    null: u64,
    code: u64,
    data: u64,
    tss_lo: u64,
    tss_hi: u64,
}

#[repr(C, align(8))]
struct Tss([u8; 0x68]);

#[repr(packed)]
struct Gdtr {
    limit: u16,
    base: u64,
}

static mut GDT: Gdt = Gdt {
    null: 0,
    code: 0,
    data: 0,
    tss_lo: 0,
    tss_hi: 0,
};

static mut TSS: Tss = Tss([0; 0x68]);

pub fn init() {
    unsafe {
        let g = addr_of_mut!(GDT);
        let tss = addr_of!(TSS) as u64;
        (*g).code = CODE64;
        (*g).data = DATA64;
        (*g).tss_lo = 0x67
            | ((tss & 0xFFFF) << 16)
            | (((tss >> 16) & 0xFF) << 32)
            | (0x89u64 << 40)
            | (((tss >> 24) & 0xFF) << 56);
        (*g).tss_hi = tss >> 32;
        let d = Gdtr {
            limit: 39,
            base: g as u64,
        };
        asm!("lgdt [{}]", in(reg) &d, options(nostack));
        let tr: u16 = 0x18;
        asm!("ltr {0:x}", in(reg) tr, options(nostack));
    }
}

pub fn gdt_base() -> u64 {
    addr_of!(GDT) as u64
}

pub fn tss_base() -> u64 {
    addr_of!(TSS) as u64
}
