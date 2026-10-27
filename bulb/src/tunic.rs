use crate::uart::Uart;
use core::fmt::Write as _;
use nectar::spin::Spinlock;
use tunic::{cap, Verb, Tunic};

static TUNIC: Spinlock<Tunic> = Spinlock::new(Tunic::new());

pub fn wear(console: &mut Uart) {
    let mut t = TUNIC.lock();
    t.wear();
    let _ = writeln!(console, "tunic:worn");
    if t.allows(tunic::BLOOM, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:leak:1:42");
        return;
    }
    let _ = writeln!(console, "tunic:deny:1:42");
    if !t.grant(cap(tunic::BLOOM, Verb::Send, 42)) {
        let _ = writeln!(console, "tunic:full");
        return;
    }
    let _ = writeln!(console, "tunic:grant:1:42");
    if t.allows(tunic::BLOOM, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:allow:1:42");
    } else {
        let _ = writeln!(console, "tunic:deaf:1:42");
        return;
    }
    if t.allows(tunic::LEAF, Verb::Send, 42) {
        let _ = writeln!(console, "tunic:leak:2:42");
    } else {
        let _ = writeln!(console, "tunic:deny:2:42");
    }
    if t.allows(tunic::BLOOM, Verb::Recv, 42) {
        let _ = writeln!(console, "tunic:leak:verb");
    } else {
        let _ = writeln!(console, "tunic:verb:ok");
    }
    if t.revoke(tunic::BLOOM, 42) && !t.allows(tunic::BLOOM, Verb::Send, 42) {
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
