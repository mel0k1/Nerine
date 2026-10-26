pub type FastFn = fn(u32, &[u8], &mut [u8]) -> u32;

pub struct Petal {
    pub func: FastFn,
    pub params: [u8; 16],
}

impl Petal {
    pub const fn new(func: FastFn, params: [u8; 16]) -> Self {
        Self { func, params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xform(_d: u32, p: &[u8], grain: &mut [u8]) -> u32 {
        for b in grain.iter_mut() {
            *b ^= p[0];
        }
        grain.len() as u32
    }

    #[test]
    fn petal_transforms_in_place() {
        let mut data = [0u8; 4];
        let petal = Petal::new(xform, [0xFF; 16]);
        let r = (petal.func)(1, &petal.params, &mut data);
        assert_eq!(r, 4);
        assert_eq!(data, [0xFF; 4]);
    }
}
