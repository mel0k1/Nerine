#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grain {
    pub offset: u32,
    pub len: u32,
}

impl Grain {
    pub const fn new(offset: u32, len: u32) -> Self {
        Self { offset, len }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        assert_eq!(core::mem::size_of::<Grain>(), 8);
        assert_eq!(core::mem::align_of::<Grain>(), 4);
    }
}
