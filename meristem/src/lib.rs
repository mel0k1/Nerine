#![cfg_attr(not(test), no_std)]

pub const SPROUTS_MAX: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Bloom,
    Leaf,
}

pub type StepFn = fn(&mut Sprout);

pub struct Sprout {
    pub kind: Kind,
    step: StepFn,
    pub turns: u32,
    pub sent: u32,
    pub dropped: u32,
    pub heard: u32,
    pub state: u32,
}

impl Sprout {
    pub const fn new(kind: Kind, step: StepFn) -> Self {
        Self {
            kind,
            step,
            turns: 0,
            sent: 0,
            dropped: 0,
            heard: 0,
            state: 0,
        }
    }
}

pub struct Meristem {
    sprouts: [Option<Sprout>; SPROUTS_MAX],
    n: usize,
    idx: usize,
    rot: u32,
}

impl Meristem {
    pub const fn new() -> Self {
        Self {
            sprouts: [const { None }; SPROUTS_MAX],
            n: 0,
            idx: 0,
            rot: 0,
        }
    }

    pub fn plant(&mut self, kind: Kind, step: StepFn) -> bool {
        if self.n >= SPROUTS_MAX {
            return false;
        }
        self.sprouts[self.n] = Some(Sprout::new(kind, step));
        self.n += 1;
        true
    }

    pub fn count(&self) -> usize {
        self.n
    }

    pub fn rotation(&self) -> u32 {
        self.rot
    }

    pub fn advance(&mut self) {
        self.rot = self.rot.wrapping_add(1);
        if self.n == 0 {
            return;
        }
        if self.idx >= self.n {
            self.idx = 0;
        }
        if let Some(s) = &mut self.sprouts[self.idx] {
            s.turns = s.turns.wrapping_add(1);
            (s.step)(s);
        }
        self.idx = (self.idx + 1) % self.n;
    }

    pub fn turn_of(&self, kind: Kind) -> u32 {
        let mut t = 0u32;
        let mut i = 0;
        while i < self.n {
            if let Some(s) = &self.sprouts[i] {
                if s.kind == kind {
                    t = t.wrapping_add(s.turns);
                }
            }
            i += 1;
        }
        t
    }

    pub fn stat_of(&self, kind: Kind) -> (u32, u32, u32, u32) {
        let mut turns = 0u32;
        let mut sent = 0u32;
        let mut dropped = 0u32;
        let mut heard = 0u32;
        let mut i = 0;
        while i < self.n {
            if let Some(s) = &self.sprouts[i] {
                if s.kind == kind {
                    turns = turns.wrapping_add(s.turns);
                    sent = sent.wrapping_add(s.sent);
                    dropped = dropped.wrapping_add(s.dropped);
                    heard = heard.wrapping_add(s.heard);
                }
            }
            i += 1;
        }
        (turns, sent, dropped, heard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static ORDER: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

    fn bump(_s: &mut Sprout) {
        CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn bloom_call(_s: &mut Sprout) {
        ORDER.lock().unwrap().push("b");
    }

    fn leaf_call(_s: &mut Sprout) {
        ORDER.lock().unwrap().push("l");
    }

    fn grow(s: &mut Sprout) {
        s.state += 2;
        if s.state % 4 == 0 {
            s.sent += 1;
        } else {
            s.dropped += 1;
        }
        s.heard += 3;
    }

    #[test]
    fn empty_plants() {
        let mut m = Meristem::new();
        assert_eq!(m.count(), 0);
        assert_eq!(m.rotation(), 0);
        assert!(m.plant(Kind::Bloom, bump));
        assert!(m.plant(Kind::Leaf, bump));
        assert_eq!(m.count(), 2);
    }

    #[test]
    fn advance_counts_rotations() {
        let mut m = Meristem::new();
        for _ in 0..7 {
            m.advance();
        }
        assert_eq!(m.rotation(), 7);
        assert_eq!(m.turn_of(Kind::Bloom), 0);
    }

    #[test]
    fn round_robin_alternates() {
        let mut m = Meristem::new();
        assert!(m.plant(Kind::Bloom, bloom_call));
        assert!(m.plant(Kind::Leaf, leaf_call));
        for _ in 0..5 {
            m.advance();
        }
        let order = ORDER.lock().unwrap();
        assert_eq!(order.len(), 5);
        assert_eq!(order[0], "b");
        assert_eq!(order[1], "l");
        assert_eq!(order[2], "b");
        assert_eq!(order[3], "l");
        assert_eq!(order[4], "b");
    }

    #[test]
    fn turns_track_per_kind() {
        let mut m = Meristem::new();
        m.plant(Kind::Bloom, bump);
        m.plant(Kind::Leaf, bump);
        for _ in 0..9 {
            m.advance();
        }
        assert_eq!(m.rotation(), 9);
        assert_eq!(m.turn_of(Kind::Bloom), 5);
        assert_eq!(m.turn_of(Kind::Leaf), 4);
    }

    #[test]
    fn empty_advance_is_harmless() {
        let mut m = Meristem::new();
        m.advance();
        m.advance();
        assert_eq!(m.rotation(), 2);
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn garden_is_bounded() {
        let mut m = Meristem::new();
        for _ in 0..SPROUTS_MAX {
            assert!(m.plant(Kind::Leaf, bump));
        }
        assert!(!m.plant(Kind::Bloom, bump));
        assert_eq!(m.count(), SPROUTS_MAX);
    }

    #[test]
    fn steps_shape_their_state() {
        let mut m = Meristem::new();
        m.plant(Kind::Bloom, grow);
        for _ in 0..8 {
            m.advance();
        }
        let (turns, sent, dropped, heard) = m.stat_of(Kind::Bloom);
        assert_eq!(turns, 8);
        assert_eq!(sent, 4);
        assert_eq!(dropped, 4);
        assert_eq!(heard, 24);
    }

    #[test]
    fn unknown_kind_reports_zero() {
        let mut m = Meristem::new();
        m.plant(Kind::Bloom, bump);
        for _ in 0..3 {
            m.advance();
        }
        assert_eq!(m.turn_of(Kind::Leaf), 0);
        assert_eq!(m.stat_of(Kind::Leaf), (0, 0, 0, 0));
    }

    #[test]
    fn single_sprout_takes_every_turn() {
        let mut m = Meristem::new();
        m.plant(Kind::Bloom, bump);
        for _ in 0..5 {
            m.advance();
        }
        assert_eq!(CALLS.load(Ordering::SeqCst), 5);
        assert_eq!(m.turn_of(Kind::Bloom), 5);
    }

    #[test]
    fn stats_of_two_kinds_stay_separate() {
        let mut m = Meristem::new();
        m.plant(Kind::Bloom, grow);
        m.plant(Kind::Leaf, bump);
        for _ in 0..4 {
            m.advance();
        }
        let (bt, bs, bd, bh) = m.stat_of(Kind::Bloom);
        assert_eq!((bt, bs, bd, bh), (2, 1, 1, 6));
        assert_eq!(m.turn_of(Kind::Leaf), 2);
    }
}
