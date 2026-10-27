pub mod shadow;
pub mod vmx;

use crate::corolla;
use crate::uart::Uart;
use core::arch::asm;
use core::fmt::Write as _;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

static mut BLOOM_DONE: bool = false;

pub fn bloom_done() -> bool {
    unsafe { addr_of!(BLOOM_DONE).read_volatile() }
}

pub fn vmx_supported() -> bool {
    vmx::supported()
}

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
    unsafe {
        addr_of_mut!(BLOOM_DONE).write_volatile(true);
    }
    crate::sepal::run()
}

pub fn main_phase() -> ! {
    let mut console = Uart::new();
    if vmx::supported() {
        let _ = vmx::launch(&mut console, vmx::Vm::Main);
        loop {
            unsafe { asm!("hlt") };
        }
    }
    shadow::run();
    terminate(&mut console);
}

pub fn terminate(console: &mut Uart) -> ! {
    let _ = writeln!(console, "guest:done");
    corolla::run()
}
