pub mod shadow;
pub mod vmx;

use crate::uart::{self, Uart};
use core::arch::asm;
use core::fmt::Write as _;

pub fn boot_guest(console: &mut Uart) -> ! {
    let has = vmx::supported();
    let _ = writeln!(console, "vmx:{}", has as u8);
    if has && vmx::enter(console).is_ok() {
        loop {
            unsafe { asm!("hlt") };
        }
    }
    shadow::run();
    terminate(console);
}

pub fn terminate(console: &mut Uart) -> ! {
    let _ = writeln!(console, "guest:done");
    uart::exit(0x1F);
    loop {
        unsafe { asm!("hlt") };
    }
}
