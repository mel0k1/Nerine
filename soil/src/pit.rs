pub const BASE_HZ: u32 = 1_193_182;

pub fn counts(ms: u32) -> u16 {
    let v = u64::from(BASE_HZ) * u64::from(ms) / 1000;
    v.min(0xFFFF) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_ms() {
        assert_eq!(counts(10), 11_931);
    }

    #[test]
    fn clamps_to_sixteen_bits() {
        assert_eq!(counts(100_000), 0xFFFF);
    }
}
