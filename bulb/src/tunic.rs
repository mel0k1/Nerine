use crate::lanes;
use crate::uart::Uart;
use core::fmt::Write as _;
use nectar::msg::Msg;
use nectar::spin::Spinlock;
use tunic::{cap, Verb, Tunic};

static TUNIC: Spinlock<Tunic> = Spinlock::new(Tunic::new());

pub fn gate(subject: u8, verb: Verb, label: u32) -> bool {
    TUNIC.lock().allows(subject, verb, label)
}

pub fn wear(console: &mut Uart) {
    TUNIC.lock().wear();
    let _ = writeln!(console, "tunic:worn");
    if TUNIC.lock().allows(tunic::BLOOM, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:leak:1:42");
        return;
    }
    let _ = writeln!(console, "tunic:deny:1:42");
    let first = Msg::new(lanes::LABEL_BLOOM, 1);
    if lanes::bloom_say(first) {
        let _ = writeln!(console, "tunic:leak:lane");
        return;
    }
    let _ = writeln!(console, "tunic:hot:deny");
    if !TUNIC.lock().grant(cap(tunic::BLOOM, Verb::Send, 42)) {
        let _ = writeln!(console, "tunic:full");
        return;
    }
    let _ = writeln!(console, "tunic:grant:1:42");
    if !TUNIC.lock().allows(tunic::BLOOM, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:deaf:1:42");
        return;
    }
    let _ = writeln!(console, "tunic:allow:1:42");
    let second = Msg::new(lanes::LABEL_BLOOM, 2);
    if !lanes::bloom_say(second) {
        let _ = writeln!(console, "tunic:deaf:lane");
        return;
    }
    let _ = writeln!(console, "tunic:hot:sent");
    TUNIC.lock().grant(cap(tunic::LEAF, Verb::Recv, 42));
    let _ = writeln!(console, "tunic:grant:2:42");
    match lanes::from_bloom() {
        Some(m) if m.label == lanes::LABEL_BLOOM => {
            let _ = writeln!(console, "tunic:hot:recv:{}", m.label);
        }
        _ => {
            let _ = writeln!(console, "tunic:deaf:recv");
            return;
        }
    }
    if TUNIC.lock().allows(tunic::LEAF, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:leak:2:42");
    } else {
        let _ = writeln!(console, "tunic:deny:2:42");
    }
    if TUNIC.lock().allows(tunic::BLOOM, Verb::Recv, 42) {
        let _ = writeln!(console, "tunic:leak:verb");
    } else {
        let _ = writeln!(console, "tunic:verb:ok");
    }
    let garden = [
        cap(tunic::LEAF, Verb::Send, lanes::LABEL_LEAF),
        cap(tunic::BLOOM, Verb::Recv, lanes::LABEL_LEAF),
        cap(tunic::LEAF, Verb::Send, lanes::LABEL_ECHO),
        cap(tunic::LEAF, Verb::Recv, lanes::LABEL_ECHO),
    ];
    let mut grown = true;
    {
        let mut t = TUNIC.lock();
        for c in garden {
            grown = grown && t.grant(c);
        }
        if t.revoke(tunic::BLOOM, 43) {
            grown = false;
        }
    }
    if !grown {
        let _ = writeln!(console, "tunic:garden:stuck");
        return;
    }
    let _ = writeln!(console, "tunic:garden:{}", TUNIC.lock().caps());
    TUNIC.lock().grant(cap(tunic::BLOOM, Verb::Send, 43));
    if TUNIC.lock().revoke(tunic::BLOOM, 43) && !TUNIC.lock().allows(tunic::BLOOM, Verb::Send, 43) {
        let _ = writeln!(console, "tunic:revoke:ok");
    } else {
        let _ = writeln!(console, "tunic:revoke:stuck");
    }
}

pub fn bare(console: &mut Uart) {
    TUNIC.lock().bare();
    let _ = writeln!(console, "tunic:bare");
}

pub fn status(console: &mut Uart) {
    let t = TUNIC.lock();
    let w = if t.worn() { "worn" } else { "bare" };
    let _ = writeln!(
        console,
        "corolla:tunic:{}:{} checks:{} deny:{}",
        w,
        t.caps(),
        t.checks(),
        t.denied()
    );
}

pub fn tunic(console: &mut Uart, arg: &[u8]) {
    match arg {
        b"on" => wear(console),
        b"off" => bare(console),
        b"" => status(console),
        _ => {
            let _ = writeln!(console, "tunic:? on|off");
        }
    }
}
