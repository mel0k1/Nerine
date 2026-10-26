use core::arch::{asm, global_asm};
use core::ptr::addr_of_mut;

global_asm!(
    ".text",
    ".globl exc_common",
    "exc_common:",
    "mov edi, [rsp]",
    "mov rsi, [rsp+8]",
    "add rsp, 8",
    "mov dx, 0x3f8",
    "mov al, 'E'",
    "out dx, al",
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
extern "C" fn fault_rust(v: usize, rip: usize) -> ! {
    crate::uart::puts(b"ex:");
    crate::uart::hex8(v as u8);
    crate::uart::puts(b" rip:");
    crate::uart::hex64(rip);
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

#[repr(packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

const STUB_FNS: [unsafe extern "C" fn(); ENTRIES] = [
    exc0, exc1, exc2, exc3, exc4, exc5, exc6, exc7, exc8, exc9, exc10, exc11, exc12, exc13, exc14,
    exc15, exc16, exc17, exc18, exc19, exc20, exc21, exc22, exc23, exc24, exc25, exc26, exc27,
    exc28, exc29, exc30, exc31, exc32, exc33, exc34, exc35, exc36, exc37, exc38, exc39, exc40,
    exc41, exc42, exc43, exc44, exc45, exc46, exc47,
];

pub fn init() {
    let base = addr_of_mut!(IDT) as *mut Entry;
    for (i, f) in STUB_FNS.iter().enumerate() {
        let f = (*f) as usize;
        let lo = (f as u64 & 0xFFFF)
            | (0x08u64 << 16)
            | (0x8Eu64 << 40)
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
