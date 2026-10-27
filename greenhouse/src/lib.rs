#![no_std]

pub const DOMAINS_MAX: usize = 8;
pub const STAGE_MAX: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Perf,
    Hard,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Path {
    ZeroCopy,
    Broker,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Bloom,
    Leaf,
}

#[derive(Clone, Copy)]
pub struct Domain {
    pub id: u8,
    pub kind: Kind,
    pub mode: Mode,
}

pub fn path(system: Mode, a: Mode, b: Mode) -> Path {
    if system == Mode::Perf && a == Mode::Perf && b == Mode::Perf {
        Path::ZeroCopy
    } else {
        Path::Broker
    }
}

pub struct Broker {
    stage: [u8; STAGE_MAX],
    used: usize,
}

impl Broker {
    pub const fn new() -> Self {
        Self {
            stage: [0; STAGE_MAX],
            used: 0,
        }
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn relay(&mut self, src: &[u8], dst: &mut [u8]) -> usize {
        let n = src.len().min(self.stage.len()).min(dst.len());
        self.stage[..n].copy_from_slice(&src[..n]);
        dst[..n].copy_from_slice(&self.stage[..n]);
        self.used = n;
        n
    }
}

pub struct Garden {
    mode: Mode,
    domains: [Option<Domain>; DOMAINS_MAX],
    n: usize,
    relays: u32,
    denied: u32,
}

impl Garden {
    pub const fn new() -> Self {
        Self {
            mode: Mode::Perf,
            domains: [None; DOMAINS_MAX],
            n: 0,
            relays: 0,
            denied: 0,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn domains(&self) -> usize {
        self.n
    }

    pub fn relays(&self) -> u32 {
        self.relays
    }

    pub fn denied(&self) -> u32 {
        self.denied
    }

    pub fn enable_hard(&mut self) {
        self.mode = Mode::Hard;
    }

    pub fn disable(&mut self) {
        self.mode = Mode::Perf;
    }

    pub fn attach(&mut self, kind: Kind, mode: Mode) -> Option<u8> {
        if self.n >= DOMAINS_MAX {
            return None;
        }
        let id = self.n as u8;
        self.domains[self.n] = Some(Domain { id, kind, mode });
        self.n += 1;
        Some(id)
    }

    pub fn transfer(&self, a: u8, b: u8) -> Option<Path> {
        let da = *self.domains.get(a as usize)?;
        let db = *self.domains.get(b as usize)?;
        Some(path(self.mode, da?.mode, db?.mode))
    }

    pub fn relayed(&mut self) {
        self.relays += 1;
    }

    pub fn deny(&mut self) {
        self.denied += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_by_default() {
        let mut g = Garden::new();
        let a = g.attach(Kind::Bloom, Mode::Perf).unwrap();
        let b = g.attach(Kind::Leaf, Mode::Perf).unwrap();
        assert_eq!(g.mode(), Mode::Perf);
        assert_eq!(g.transfer(a, b), Some(Path::ZeroCopy));
    }

    #[test]
    fn opt_in_hard_brokered() {
        let mut g = Garden::new();
        let a = g.attach(Kind::Bloom, Mode::Perf).unwrap();
        let b = g.attach(Kind::Leaf, Mode::Perf).unwrap();
        g.enable_hard();
        assert_eq!(g.transfer(a, b), Some(Path::Broker));
        g.disable();
        assert_eq!(g.transfer(a, b), Some(Path::ZeroCopy));
    }

    #[test]
    fn hard_domain_opts_out_of_sharing() {
        let mut g = Garden::new();
        let a = g.attach(Kind::Bloom, Mode::Hard).unwrap();
        let b = g.attach(Kind::Leaf, Mode::Perf).unwrap();
        assert_eq!(g.transfer(a, b), Some(Path::Broker));
        assert_eq!(g.transfer(b, b), Some(Path::ZeroCopy));
    }

    #[test]
    fn unknown_ids_denied() {
        let mut g = Garden::new();
        let a = g.attach(Kind::Bloom, Mode::Perf).unwrap();
        assert_eq!(g.transfer(a, 99), None);
        assert_eq!(g.transfer(99, a), None);
    }

    #[test]
    fn registry_is_bounded() {
        let mut g = Garden::new();
        for _ in 0..DOMAINS_MAX {
            assert!(g.attach(Kind::Bloom, Mode::Perf).is_some());
        }
        assert_eq!(g.attach(Kind::Bloom, Mode::Perf), None);
        assert_eq!(g.domains(), DOMAINS_MAX);
    }

    #[test]
    fn broker_relay_copies_through_stage() {
        let mut broker = Broker::new();
        let src = b"greenhouse";
        let mut dst = [0u8; 16];
        let n = broker.relay(src, &mut dst);
        assert_eq!(n, 10);
        assert_eq!(&dst[..n], src);
        assert_eq!(broker.used(), 10);
        assert_ne!(&dst[..], &src[..]);
        assert_eq!(&dst[..n], &src[..]);
    }

    #[test]
    fn broker_truncates_to_stage() {
        let mut broker = Broker::new();
        let src = [7u8; STAGE_MAX + 10];
        let mut dst = [0u8; STAGE_MAX + 10];
        let n = broker.relay(&src, &mut dst);
        assert_eq!(n, STAGE_MAX);
        assert_eq!(&dst[..n], &src[..n]);
        assert_eq!(broker.used(), STAGE_MAX);
    }

    #[test]
    fn broker_respects_small_dst() {
        let mut broker = Broker::new();
        let src = [3u8; 100];
        let mut dst = [0u8; 32];
        let n = broker.relay(&src, &mut dst);
        assert_eq!(n, 32);
        assert_eq!(&dst[..], &src[..32]);
    }

    #[test]
    fn counters() {
        let mut g = Garden::new();
        assert_eq!((g.relays(), g.denied()), (0, 0));
        g.relayed();
        g.deny();
        g.deny();
        assert_eq!((g.relays(), g.denied()), (1, 2));
    }
}
