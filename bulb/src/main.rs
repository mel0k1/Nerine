#![no_std]
#![no_main]

mod boot;
mod corolla;
mod gdt;
mod greenhouse;
mod ground;
mod hyp;
mod idt;
mod lanes;
mod mem;
mod sepal;
mod tunic;
mod uart;

use core::arch::asm;
use core::fmt::Write as _;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use uart::Uart;

extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        let mut p = addr_of!(__bss_start) as *mut u8;
        let end = addr_of!(__bss_end) as *mut u8;
        while p < end {
            p.write_volatile(0);
            p = p.add(1);
        }
        let mut cr0: u64;
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack));
        asm!("mov cr0, {}", in(reg) cr0 & !0x4, options(nostack));
        let mut cr4: u64;
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));
        asm!("mov cr4, {}", in(reg) cr4 | 0x200, options(nostack));
    }
    let mut console = Uart::new();
    let _ = writeln!(console, "Nerine / Bulb 0.1.0 (bulb)");
    idt::init();
    gdt::init();
    if !boot::ready() || !mem::init() {
        let _ = writeln!(console, "boot:no-memmap");
        halt();
    }
    corolla::plant(&mut console);
    ground::init(&mut console);
    hyp::boot_all();
}

fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut console = Uart::new();
    let _ = writeln!(console, "panic: {info}");
    halt()
}
