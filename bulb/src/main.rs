#![no_std]
#![no_main]

mod uart;

use core::arch::asm;
use core::fmt::Write as _;
use core::panic::PanicInfo;
use uart::Uart;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut console = Uart::new();
    let _ = writeln!(console, "Nerine / Bulb 0.1.0 (stem)");
    let _ = writeln!(console, "one bulb, many blooms");
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut console = Uart::new();
    let _ = writeln!(console, "panic: {info}");
    loop {
        unsafe { asm!("hlt") };
    }
}
