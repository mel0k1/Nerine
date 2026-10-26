#![no_std]
#![no_main]

mod uart;

use core::arch::asm;
use core::fmt::Write as _;
use core::panic::PanicInfo;
use nectar::msg::Msg;
use nectar::spsc::Lane;
use uart::Uart;

static LANE: Lane<4> = Lane::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut console = Uart::new();
    let _ = writeln!(console, "Nerine / Bulb 0.1.0 (nectar)");
    let sent = LANE.try_send(Msg::new(7, 1)).is_ok();
    let got = LANE.try_recv().map(|m| m.label).unwrap_or(0);
    let _ = writeln!(console, "nectar:{}:{}", sent as u8, got);
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
