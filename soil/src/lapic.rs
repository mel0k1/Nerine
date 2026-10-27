pub const EOI: u32 = 0x0B0;
pub const SVR: u32 = 0x0F0;
pub const TIMER_LVT: u32 = 0x320;
pub const TIMER_ICR: u32 = 0x380;
pub const TIMER_CCR: u32 = 0x390;
pub const TIMER_DCR: u32 = 0x3E0;
pub const DIV_16: u32 = 0x8;

pub fn x2_msr(off: u32) -> u32 {
    0x800 | (off >> 4)
}

pub fn svr(vector: u8) -> u32 {
    u32::from(vector) | 0x100
}

pub fn lvt_periodic(vector: u8) -> u32 {
    u32::from(vector) | 1 << 17
}

pub fn lvt_masked(vector: u8) -> u32 {
    u32::from(vector) | 1 << 16
}

pub fn reload(per_10ms: u64, rate: u64) -> u32 {
    (per_10ms * 100 / rate).max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x2_msr_maps_offsets() {
        assert_eq!(x2_msr(EOI), 0x80B);
        assert_eq!(x2_msr(SVR), 0x80F);
        assert_eq!(x2_msr(TIMER_LVT), 0x832);
        assert_eq!(x2_msr(TIMER_ICR), 0x838);
        assert_eq!(x2_msr(TIMER_CCR), 0x839);
        assert_eq!(x2_msr(TIMER_DCR), 0x83E);
    }

    #[test]
    fn encodings() {
        assert_eq!(svr(0x3F), 0x13F);
        assert_eq!(lvt_periodic(0x20), 0x20020);
        assert_eq!(lvt_masked(0x20), 0x10020);
    }

    #[test]
    fn reload_math() {
        assert_eq!(reload(62_500, 128), 48_828);
        assert_eq!(reload(0, 128), 1);
    }
}
