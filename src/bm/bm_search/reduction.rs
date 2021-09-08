#[derive(Debug, Clone)]
pub struct Reduction {
    base: u32,
    factor: u32,
    divisor: u32,
}

impl Reduction {
    pub const fn new(base: u32, factor: u32, divisor: u32) -> Self {
        Self {
            base,
            factor,
            divisor,
        }
    }

    #[inline]
    pub const fn reduction(&self, depth: u32) -> u32 {
        self.base + (depth * self.factor) / self.divisor
    }
}
