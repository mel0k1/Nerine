use crate::ground;
use crate::hyp;
use crate::uart::{self, Uart};
use core::arch::asm;
use core::fmt::Write as _;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use nectar::msg::Msg;
use nectar::spsc::Lane;
use pollen::fastpath::Petal;
use pollen::ring::Ring;

const LINE_MAX: usize = 32;
const LANE_CAP: usize = 4;
const RING_CAP: u32 = 4;
const RING_SLOT: u32 = 64;
const BED_LEN: usize = 512;

#[repr(align(8))]
struct Bed([u8; BED_LEN]);

static mut BED: Bed = Bed([0; BED_LEN]);
static mut LAUNCHES: u32 = 0;

static LANE: Lane<LANE_CAP> = Lane::new();

pub fn launch_bump() {
    unsafe {
        let p = addr_of_mut!(LAUNCHES);
        p.write_volatile(p.read_volatile() + 1);
    }
}

fn launches() -> u32 {
    unsafe { addr_of!(LAUNCHES).read_volatile() }
}

fn fast_sum(_d: u32, _p: &[u8], g: &mut [u8]) -> u32 {
    g.iter().map(|b| *b as u32).sum()
}

pub fn plant(console: &mut Uart) {
    let sent = LANE.try_send(Msg::new(7, 1)).is_ok();
    let got = LANE.try_recv().map(|m| m.label).unwrap_or(0);
    let _ = writeln!(console, "nectar:{}:{}", sent as u8, got);
    let bed = addr_of_mut!(BED);
    let bed_buf = unsafe { &mut *addr_of_mut!((*bed).0) };
    let ring = match Ring::init(bed_buf, RING_CAP, RING_SLOT, Some(Petal::new(fast_sum, [0; 16]))) {
        Ok(r) => r,
        Err(_) => {
            let _ = writeln!(console, "pollen:0:0");
            return;
        }
    };
    let l = match ring.grant() {
        Some(g) => {
            g[..5].copy_from_slice(b"polln");
            5u32
        }
        None => 0,
    };
    let published = l == 5 && ring.publish(l);
    let got = ring.poll().map(|gr| ring.data(&gr).len()).unwrap_or(0);
    ring.release();
    let _ = writeln!(console, "pollen:{}:{}", (published && got == 5) as u8, ring.fast_result());
}

fn split(line: &[u8]) -> (&[u8], &[u8]) {
    let n = line.iter().position(|b| *b == b' ').unwrap_or(line.len());
    let rest = if n < line.len() { &line[n + 1..] } else { &line[n..] };
    (&line[..n], rest)
}

fn cmd_help(console: &mut Uart) {
    let _ = writeln!(console, "corolla:help");
    let _ = writeln!(console, "help soil uptime nectar pollen bloom vm exit");
}

fn cmd_soil(console: &mut Uart) {
    let _ = writeln!(
        console,
        "corolla:soil:apic={} x2={} ticks={} hz={}",
        ground::apic_on() as u8,
        ground::x2() as u8,
        ground::ticks(),
        ground::HZ
    );
}

fn cmd_uptime(console: &mut Uart) {
    let ms = u64::from(ground::ticks()) * 1000 / ground::HZ;
    let _ = writeln!(console, "uptime:{}.{}s", ms / 1000, ms % 1000);
}

fn cmd_nectar(console: &mut Uart) {
    let sent = LANE.try_send(Msg::new(11, 2)).is_ok();
    let got = LANE.try_recv().map(|m| m.label).unwrap_or(0);
    let _ = writeln!(console, "corolla:nectar:cap={}", LANE_CAP);
    let _ = writeln!(console, "corolla:nectar:sent={} label={}", sent as u8, got);
}

fn cmd_pollen(console: &mut Uart) {
    let bed = addr_of!(BED);
    let buf = unsafe { core::slice::from_raw_parts(bed as *const u8, BED_LEN) };
    match Ring::attach(buf) {
        Ok(r) => {
            let _ = writeln!(console, "corolla:pollen:cap={} slot={}", r.capacity(), r.slot_size());
            let _ = writeln!(console, "corolla:pollen:bell={} fast={}", r.doorbell(), r.fast_result());
        }
        Err(_) => {
            let _ = writeln!(console, "corolla:pollen:none");
        }
    }
}

fn cmd_bloom(console: &mut Uart) {
    if hyp::bloom_done() {
        let _ = writeln!(console, "corolla:bloom:console:ok");
    } else {
        let _ = writeln!(console, "corolla:bloom:none");
    }
}

fn cmd_vm(console: &mut Uart) {
    let _ = writeln!(console, "corolla:vmx:{} vms={}", hyp::vmx_supported() as u8, launches());
}

fn bye() -> ! {
    let mut console = Uart::bare();
    let _ = writeln!(console, "corolla:bye");
    unsafe { asm!("cli", options(nostack)) };
    uart::exit(0x1F);
    loop {
        unsafe { asm!("hlt") };
    }
}

fn dispatch(console: &mut Uart, line: &[u8]) {
    let (cmd, _arg) = split(line);
    unsafe { asm!("cli", options(nostack)) };
    match cmd {
        b"help" => cmd_help(console),
        b"soil" => cmd_soil(console),
        b"uptime" => cmd_uptime(console),
        b"nectar" => cmd_nectar(console),
        b"pollen" => cmd_pollen(console),
        b"bloom" => cmd_bloom(console),
        b"vm" => cmd_vm(console),
        b"exit" => bye(),
        _ => {
            uart::puts(b"corolla:?");
            uart::puts(line);
            uart::put(b'\n');
        }
    }
    if ground::sleep_ok() {
        unsafe { asm!("sti", options(nostack)) };
    }
}

pub fn run() -> ! {
    let mut console = Uart::bare();
    ground::serve();
    let _ = writeln!(console, "corolla:ready");
    let mut line = [0u8; LINE_MAX];
    loop {
        uart::puts(b"corolla> ");
        let mut n = 0usize;
        loop {
            match uart::get() {
                Some(b'\r') | Some(b'\n') => {
                    let _ = writeln!(console);
                    break;
                }
                Some(0x08) | Some(0x7F) => {
                    if n > 0 {
                        n -= 1;
                        uart::puts(b"\x08 \x08");
                    }
                }
                Some(0x15) => {
                    while n > 0 {
                        n -= 1;
                        uart::puts(b"\x08 \x08");
                    }
                }
                Some(0x03) => {
                    n = 0;
                    uart::puts(b"^C\n");
                    break;
                }
                Some(c) if (0x20..=0x7E).contains(&c) && n < LINE_MAX => {
                    line[n] = c;
                    n += 1;
                    uart::put(c);
                }
                Some(_) => {}
                None => unsafe {
                    if ground::sleep_ok() {
                        asm!("hlt");
                    }
                },
            }
        }
        if n > 0 {
            dispatch(&mut console, &line[..n]);
        }
    }
}
