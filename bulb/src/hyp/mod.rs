pub mod shadow;
pub mod vmx;

use crate::uart::{self, Uart};
use core::arch::asm;
use core::fmt::Write as _;

pub fn boot_all() -> ! {
    let mut console = Uart::new();
    let has = vmx::supported();
    let _ = writeln!(console, "vmx:{}", has as u8);
    if has {
        let _ = vmx::launch(&mut console, vmx::Vm::Bloom);
    }
    shadow::run_bloom();
    after_bloom();
}

pub fn after_bloom() -> ! {
    crate::sepal::run()
}

pub fn main_phase() -> ! {
    let mut console = Uart::new();
    if vmx::supported() && vmx::launch(&mut console, vmx::Vm::Main).is_ok() {
        loop {
            unsafe { asm!("hlt") };
        }
    }
    shadow::run();
    terminate(&mut console);
}

pub fn terminate(console: &mut Uart) -> ! {
    let _ = writeln!(console, "guest:done");
    uart::exit(0x1F);
    loop {
        unsafe { asm!("hlt") };
    }
}
