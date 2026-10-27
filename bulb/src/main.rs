#![no_std]
#![no_main]

mod boot;
mod gdt;
mod ground;
mod hyp;
mod idt;
mod mem;
mod sepal;
mod uart;

use core::arch::asm;
use core::fmt::Write as _;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use nectar::msg::Msg;
use nectar::spsc::Lane;
use pollen::fastpath::Petal;
use pollen::ring::Ring;
use uart::Uart;

static LANE: Lane<4> = Lane::new();

#[repr(align(8))]
struct Bed([u8; 512]);

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
    let sent = LANE.try_send(Msg::new(7, 1)).is_ok();
    let got = LANE.try_recv().map(|m| m.label).unwrap_or(0);
    let _ = writeln!(console, "nectar:{}:{}", sent as u8, got);
    let mut bed = Bed([0; 512]);
    let (polled, fast) = match Ring::init(&mut bed.0, 4, 64, Some(Petal::new(fast_sum, [0; 16]))) {
        Ok(r) => {
            let l = match r.grant() {
                Some(g) => {
                    g[..5].copy_from_slice(b"polln");
                    5u32
                }
                None => 0,
            };
            let published = l == 5 && r.publish(l);
            let got = r.poll().map(|gr| r.data(&gr).len()).unwrap_or(0);
            r.release();
            (published && got == 5, r.fast_result())
        }
        Err(_) => (false, 0),
    };
    let _ = writeln!(console, "pollen:{}:{}", polled as u8, fast);
    ground::init(&mut console);
    hyp::boot_all();
}

fn fast_sum(_d: u32, _p: &[u8], g: &mut [u8]) -> u32 {
    g.iter().map(|b| *b as u32).sum()
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
