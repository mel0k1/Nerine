#![no_std]
#![no_main]

mod boot;
mod gdt;
mod hyp;
mod idt;
mod mem;
mod uart;

use core::arch::asm;
use core::fmt::Write as _;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use nectar::msg::Msg;
use nectar::spsc::Lane;
use uart::Uart;

static LANE: Lane<4> = Lane::new();

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
    let _ = console.write_str("i\n");
    gdt::init();
    let _ = console.write_str("g\n");
    if !boot::ready() || !mem::init() {
        let _ = writeln!(console, "boot:no-memmap");
        halt();
    }
    let _ = console.write_str("m\n");
    let sent = LANE.try_send(Msg::new(7, 1)).is_ok();
    let _ = console.write_str("s\n");
    let got = LANE.try_recv().map(|m| m.label).unwrap_or(0);
    let _ = console.write_str("r\n");
    let _ = writeln!(console, "nectar:{}:{}", sent as u8, got);
    hyp::boot_guest(&mut console);
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
