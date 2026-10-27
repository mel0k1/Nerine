#![no_std]

pub const CAPS_MAX: usize = 16;

pub const BLOOM: u8 = 1;
pub const LEAF: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verb {
    Send,
    Recv,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cap {
    pub subject: u8,
    pub verb: Verb,
    pub label: u32,
}

pub const fn cap(subject: u8, verb: Verb, label: u32) -> Cap {
    Cap {
        subject,
        verb,
        label,
    }
}

pub struct Tunic {
    caps: [Option<Cap>; CAPS_MAX],
    n: usize,
    worn: bool,
    checks: u32,
    denied: u32,
}

impl Tunic {
    pub const fn new() -> Self {
        Self {
            caps: [None; CAPS_MAX],
            n: 0,
            worn: false,
            checks: 0,
            denied: 0,
        }
    }

    pub fn worn(&self) -> bool {
        self.worn
    }

    pub fn caps(&self) -> usize {
        self.n
    }

    pub fn checks(&self) -> u32 {
        self.checks
    }

    pub fn denied(&self) -> u32 {
        self.denied
    }

    pub fn wear(&mut self) {
        self.worn = true;
    }

    pub fn bare(&mut self) {
        self.worn = false;
    }

    pub fn grant(&mut self, c: Cap) -> bool {
        for i in 0..self.n {
            match self.caps[i] {
                Some(held)
                    if held.subject == c.subject
                        && held.verb == c.verb
                        && held.label == c.label =>
                {
                    return true;
                }
                _ => {}
            }
        }
        if self.n >= CAPS_MAX {
            return false;
        }
        self.caps[self.n] = Some(c);
        self.n += 1;
        true
    }

    pub fn revoke(&mut self, subject: u8, label: u32) -> bool {
        let mut removed = false;
        let mut i = 0usize;
        while i < self.n {
            match self.caps[i] {
                Some(held) if held.subject == subject && held.label == label => {
                    self.n -= 1;
                    self.caps[i] = self.caps[self.n];
                    self.caps[self.n] = None;
                    removed = true;
                }
                _ => i += 1,
            }
        }
        removed
    }

    pub fn allows(&mut self, subject: u8, verb: Verb, label: u32) -> bool {
        self.checks += 1;
        if !self.worn {
            return true;
        }
        for i in 0..self.n {
            match self.caps[i] {
                Some(held)
                    if held.subject == subject
                        && held.verb == verb
                        && held.label == label =>
                {
                    return true;
                }
                _ => {}
            }
        }
        self.denied += 1;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_allows_everything() {
        let mut t = Tunic::new();
        assert!(!t.worn());
        assert!(t.allows(BLOOM, Verb::Send, 42));
        assert!(t.allows(LEAF, Verb::Recv, 7));
    }

    #[test]
    fn worn_denies_by_default() {
        let mut t = Tunic::new();
        t.wear();
        assert!(t.worn());
        assert!(!t.allows(BLOOM, Verb::Send, 42));
        assert_eq!(t.denied(), 1);
    }

    #[test]
    fn grant_allows_matching() {
        let mut t = Tunic::new();
        t.wear();
        assert!(t.grant(cap(BLOOM, Verb::Send, 42)));
        assert!(t.allows(BLOOM, Verb::Send, 42));
        assert_eq!(t.denied(), 0);
    }

    #[test]
    fn wrong_verb_denied() {
        let mut t = Tunic::new();
        t.wear();
        t.grant(cap(BLOOM, Verb::Send, 42));
        assert!(!t.allows(BLOOM, Verb::Recv, 42));
    }

    #[test]
    fn wrong_subject_denied() {
        let mut t = Tunic::new();
        t.wear();
        t.grant(cap(BLOOM, Verb::Send, 42));
        assert!(!t.allows(LEAF, Verb::Send, 42));
    }

    #[test]
    fn wrong_label_denied() {
        let mut t = Tunic::new();
        t.wear();
        t.grant(cap(BLOOM, Verb::Send, 42));
        assert!(!t.allows(BLOOM, Verb::Send, 43));
    }

    #[test]
    fn revoke_removes_every_verb() {
        let mut t = Tunic::new();
        t.wear();
        t.grant(cap(BLOOM, Verb::Send, 42));
        t.grant(cap(BLOOM, Verb::Recv, 42));
        assert_eq!(t.caps(), 2);
        assert!(t.revoke(BLOOM, 42));
        assert_eq!(t.caps(), 0);
        assert!(!t.allows(BLOOM, Verb::Send, 42));
        assert!(!t.allows(BLOOM, Verb::Recv, 42));
        assert!(!t.revoke(BLOOM, 42));
    }

    #[test]
    fn duplicate_grant_stays_single() {
        let mut t = Tunic::new();
        t.wear();
        assert!(t.grant(cap(BLOOM, Verb::Send, 42)));
        assert!(t.grant(cap(BLOOM, Verb::Send, 42)));
        assert_eq!(t.caps(), 1);
    }

    #[test]
    fn registry_is_bounded() {
        let mut t = Tunic::new();
        for l in 0..CAPS_MAX as u32 {
            assert!(t.grant(cap(BLOOM, Verb::Send, l)));
        }
        assert!(!t.grant(cap(BLOOM, Verb::Send, 999)));
        assert_eq!(t.caps(), CAPS_MAX);
    }

    #[test]
    fn bare_ignores_caps_but_counts() {
        let mut t = Tunic::new();
        t.wear();
        t.grant(cap(BLOOM, Verb::Send, 42));
        t.bare();
        assert!(t.allows(LEAF, Verb::Send, 42));
        assert_eq!(t.checks(), 1);
        assert_eq!(t.denied(), 0);
    }

    #[test]
    fn counters_track() {
        let mut t = Tunic::new();
        t.wear();
        let _ = t.allows(BLOOM, Verb::Send, 1);
        let _ = t.allows(LEAF, Verb::Recv, 1);
        t.grant(cap(LEAF, Verb::Recv, 1));
        let _ = t.allows(LEAF, Verb::Recv, 1);
        assert_eq!(t.checks(), 3);
        assert_eq!(t.denied(), 2);
        assert_eq!(t.caps(), 1);
    }
}
