pub fn adjust(allowed0: u32, allowed1: u32, desired: u32) -> u32 {
    (!allowed0 | desired) & allowed1
}

pub fn fixed(cur: u64, fixed0: u64, fixed1: u64) -> u64 {
    (cur | fixed0) & fixed1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjust_forces_must_be_one_bits() {
        assert_eq!(adjust(0xFFFF_FFF0, 0xFFFF_FFFF, 0), 0xF);
    }

    #[test]
    fn adjust_clears_forbidden_bits() {
        assert_eq!(adjust(0xFFFF_FFFF, 0xFFFF_FFFE, 1), 0);
    }

    #[test]
    fn adjust_keeps_free_bits() {
        assert_eq!(adjust(0xFFFF_FFFF, 0xFFFF_FFFF, 0xA5), 0xA5);
    }

    #[test]
    fn adjust_mixed_msrs() {
        assert_eq!(adjust(0x0000_0000, 0x8000_0000, 0xFFFF_FFFF), 0x8000_0000);
        assert_eq!(adjust(0x7FFF_FFFF, 0xFFFF_FFFF, 0x1234_5678), 0x9234_5678);
    }

    #[test]
    fn fixed_forces_and_masks() {
        assert_eq!(fixed(0, 0x21, 0x25), 0x21);
        assert_eq!(fixed(0x8001_0033, 0x21, 0x25), 0x21);
    }

    #[test]
    fn fixed_keeps_free_bits() {
        assert_eq!(fixed(0x42, 0x01, 0x43), 0x43);
        assert_eq!(fixed(0x04, 0x21, 0x25), 0x25);
    }
}
