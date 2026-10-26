#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Msg {
    pub label: u32,
    pub seq: u64,
    pub payload: [u8; 16],
}

impl Msg {
    pub const fn new(label: u32, seq: u64) -> Self {
        Self {
            label,
            seq,
            payload: [0; 16],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        assert_eq!(core::mem::size_of::<Msg>(), 32);
        assert_eq!(core::mem::align_of::<Msg>(), 8);
    }
}
