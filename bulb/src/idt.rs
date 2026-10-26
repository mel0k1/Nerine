use core::arch::{asm, global_asm};
use core::ptr::addr_of_mut;

global_asm!(
    ".intel_syntax noprefix",
    ".text",
    ".globl exc_common",
    "exc_common:",
    "mov edi, [rsp]",
    "add rsp, 8",
    "call fault_rust",
);

macro_rules! stubs {
    ($($v:literal)*) => {
        $(
            global_asm!(concat!(".globl exc", stringify!($v)), concat!("exc", stringify!($v), ":"), concat!("push ", stringify!($v)), "jmp exc_common");
        )*
    };
}

stubs!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47
);

extern "C" {
    fn exc0();
    fn exc1();
    fn exc2();
    fn exc3();
    fn exc4();
    fn exc5();
    fn exc6();
    fn exc7();
    fn exc8();
    fn exc9();
    fn exc10();
    fn exc11();
    fn exc12();
    fn exc13();
    fn exc14();
    fn exc15();
    fn exc16();
    fn exc17();
    fn exc18();
    fn exc19();
    fn exc20();
    fn exc21();
    fn exc22();
    fn exc23();
    fn exc24();
    fn exc25();
    fn exc26();
    fn exc27();
    fn exc28();
    fn exc29();
    fn exc30();
    fn exc31();
    fn exc32();
    fn exc33();
    fn exc34();
    fn exc35();
    fn exc36();
    fn exc37();
    fn exc38();
    fn exc39();
    fn exc40();
    fn exc41();
    fn exc42();
    fn exc43();
    fn exc44();
    fn exc45();
    fn exc46();
    fn exc47();
}

#[no_mangle]
extern "C" fn fault_rust(v: usize) -> ! {
    crate::uart::puts(b"ex:");
    crate::uart::hex8(v as u8);
    crate::uart::put(b'\n');
    loop {
        unsafe { asm!("hlt") }
    }
}

const ENTRIES: usize = 48;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Entry([u64; 2]);

static mut IDT: [Entry; ENTRIES] = [Entry([0; 2]); ENTRIES];

#[repr(C, align(8))]
struct Idtr {
    limit: u16,
    base: u64,
}

#[rustfmt::skip]
const STUBS: [usize; ENTRIES] = [
    exc0 as usize,  exc1 as usize,  exc2 as usize,  exc3 as usize,
    exc4 as usize,  exc5 as usize,  exc6 as usize,  exc7 as usize,
    exc8 as usize,  exc9 as usize,  exc10 as usize, exc11 as usize,
    exc12 as usize, exc13 as usize, exc14 as usize, exc15 as usize,
    exc16 as usize, exc17 as usize, exc18 as usize, exc19 as usize,
    exc20 as usize, exc21 as usize, exc22 as usize, exc23 as usize,
    exc24 as usize, exc25 as usize, exc26 as usize, exc27 as usize,
    exc28 as usize, exc29 as usize, exc30 as usize, exc31 as usize,
    exc32 as usize, exc33 as usize, exc34 as usize, exc35 as usize,
    exc36 as usize, exc37 as usize, exc38 as usize, exc39 as usize,
    exc40 as usize, exc41 as usize, exc42 as usize, exc43 as usize,
    exc44 as usize, exc45 as usize, exc46 as usize, exc47 as usize,
];

pub fn init() {
    let base = addr_of_mut!(IDT) as *mut Entry;
    for (i, f) in STUBS.iter().enumerate() {
        let lo = (f as u64 & 0xFFFF)
            | (0x08u64 << 16)
            | (0x8E00u64 << 40)
            | ((f as u64 >> 16) & 0xFFFF) << 48;
        let hi = f as u64 >> 32;
        unsafe {
            (*base.add(i)).0 = [lo, hi];
        }
    }
    let d = Idtr {
        limit: (ENTRIES * 16 - 1) as u16,
        base: base as u64,
    };
    unsafe {
        asm!("lidt [{}]", in(reg) &d, options(nostack));
    }
}

pub fn idt_base() -> u64 {
    addr_of_mut!(IDT) as u64
}
