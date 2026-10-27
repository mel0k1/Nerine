use crate::ground;
use crate::lanes;
use crate::uart::Uart;
use core::fmt::Write as _;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use meristem::{Kind, Meristem, Sprout};
use nectar::msg::Msg;
use nectar::spin::Spinlock;

const HEARTBEAT: u32 = 4;
const ALIVE_ROT: u32 = 8;
const CHATTER_ROT: u32 = 128;

static GROW: Spinlock<Meristem> = Spinlock::new(Meristem::new());
static mut LAST: u32 = 0;
static mut CHATTER: bool = false;

fn bloom_step(s: &mut Sprout) {
    while lanes::from_leaf().is_some() {
        s.heard += 1;
    }
    s.state += 1;
    if s.state % HEARTBEAT == 0 {
        let msg = Msg::new(lanes::LABEL_BLOOM, u64::from(s.turns));
        if lanes::bloom_say(msg) {
            s.sent += 1;
        } else {
            s.dropped += 1;
        }
    }
}

fn leaf_step(s: &mut Sprout) {
    while lanes::from_bloom().is_some() {
        s.heard += 1;
    }
    s.state += 1;
    if s.state % HEARTBEAT == 0 {
        let msg = Msg::new(lanes::LABEL_LEAF, u64::from(s.turns));
        if lanes::leaf_say(msg) {
            s.sent += 1;
        } else {
            s.dropped += 1;
        }
    }
}

pub fn start() {
    let mut g = GROW.lock();
    g.plant(Kind::Bloom, bloom_step);
    g.plant(Kind::Leaf, leaf_step);
}

pub fn pump() {
    let now = ground::ticks();
    if now == unsafe { addr_of!(LAST).read_volatile() } {
        return;
    }
    unsafe {
        addr_of_mut!(LAST).write_volatile(now);
    }
    let rot;
    let say;
    {
        let mut g = GROW.lock();
        g.advance();
        rot = g.rotation();
        say = unsafe { addr_of!(CHATTER).read_volatile() } && rot % CHATTER_ROT == 0;
        if say {
            let mut console = Uart::bare();
            let _ = writeln!(
                console,
                "sched:rot:{}:b:{}:l:{}",
                rot,
                g.turn_of(Kind::Bloom),
                g.turn_of(Kind::Leaf)
            );
        }
    }
    if rot == ALIVE_ROT {
        let mut console = Uart::bare();
        let _ = writeln!(console, "sched:alive:{}", rot);
    }
}

pub fn sched(console: &mut Uart, arg: &[u8]) {
    match arg {
        b"on" => {
            unsafe {
                addr_of_mut!(CHATTER).write_volatile(true);
            }
            let _ = writeln!(console, "sched:on");
            let g = GROW.lock();
            let _ = writeln!(
                console,
                "sched:rot:{}:b:{}:l:{}",
                g.rotation(),
                g.turn_of(Kind::Bloom),
                g.turn_of(Kind::Leaf)
            );
        }
        b"off" => {
            unsafe {
                addr_of_mut!(CHATTER).write_volatile(false);
            }
            let _ = writeln!(console, "sched:off");
        }
        b"" => {
            let g = GROW.lock();
            let (bt, bs, bd, _) = g.stat_of(Kind::Bloom);
            let (lt, ls, ld, _) = g.stat_of(Kind::Leaf);
            let _ = writeln!(
                console,
                "corolla:sched:rot:{} bloom:{}:{}:{} leaf:{}:{}:{}",
                g.rotation(),
                bt,
                bs,
                bd,
                lt,
                ls,
                ld
            );
        }
        _ => {
            let _ = writeln!(console, "sched:? on|off");
        }
    }
}
