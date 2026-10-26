#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
