use crate::hyp;
use crate::mem;
use crate::uart::Uart;
use core::fmt::Write as _;
use greenhouse::{Broker, Garden, Kind, Mode, Path};
use nectar::spin::Spinlock;

static GARDEN: Spinlock<Garden> = Spinlock::new(Garden::new());

fn broker_demo() -> usize {
    let Some(src_frame) = mem::zalloc() else {
        return 0;
    };
    let Some(dst_frame) = mem::zalloc() else {
        return 0;
    };
    let mut broker = Broker::new();
    let src = b"gh:pollen";
    unsafe {
        let dst = mem::hhdm_virt(dst_frame) as *mut u8;
        core::ptr::write_bytes(dst, 0, 16);
        let dsts = core::slice::from_raw_parts_mut(dst, 16);
        let n = broker.relay(src, dsts);
        let ok = n == src.len() && dsts[..n] == src[..];
        mem::hhdm_virt(src_frame);
        if ok {
            n
        } else {
            0
        }
    }
}

pub fn on(console: &mut Uart) {
    let mut g = GARDEN.lock();
    g.enable_hard();
    let _ = writeln!(console, "greenhouse:hard");
    let b = g.attach(Kind::Bloom, Mode::Perf);
    let l = g.attach(Kind::Leaf, Mode::Perf);
    match (b, l) {
        (Some(b), Some(l)) => {
            let _ = writeln!(console, "corolla:gh:domains:{}", g.domains());
            if g.transfer(b, l) == Some(Path::Broker) {
                let n = broker_demo();
                if n > 0 {
                    g.relayed();
                    let _ = writeln!(console, "greenhouse:copy:{}", n);
                } else {
                    let _ = writeln!(console, "greenhouse:copy:0");
                }
            }
            if g.transfer(b, l) != Some(Path::ZeroCopy) {
                g.deny();
                let _ = writeln!(console, "greenhouse:deny:zerocopy");
            }
        }
        _ => {
            let _ = writeln!(console, "greenhouse:full");
        }
    }
}

pub fn off(console: &mut Uart) {
    GARDEN.lock().disable();
    let _ = writeln!(console, "greenhouse:perf");
}

pub fn status(console: &mut Uart) {
    let g = GARDEN.lock();
    let m = match g.mode() {
        Mode::Hard => "hard",
        Mode::Perf => "perf",
    };
    let _ = writeln!(
        console,
        "corolla:gh:{}:{} relays:{} deny:{}",
        m,
        g.domains(),
        g.relays(),
        g.denied()
    );
}

pub fn vm(console: &mut Uart) {
    let id = GARDEN.lock().attach(Kind::Bloom, Mode::Hard);
    match id {
        Some(id) => {
            let _ = writeln!(console, "gh:vm:domain:{}", id);
            hyp::hard_vm(console);
        }
        None => {
            let _ = writeln!(console, "greenhouse:full");
        }
    }
}

pub fn gh(console: &mut Uart, arg: &[u8]) {
    match arg {
        b"on" => on(console),
        b"off" => off(console),
        b"vm" => vm(console),
        b"" => status(console),
        _ => {
            let _ = writeln!(console, "gh:? on|off|vm");
        }
    }
}
