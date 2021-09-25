#[derive(Debug, Clone)]
pub struct Threshold {
    base: i16,
    factor: i16,
}

impl Threshold {
    pub const fn new(base: i16, factor: i16) -> Self {
        Self { base, factor }
    }

    #[inline]
    pub fn threshold(&self, depth_left: u32) -> i16 {
        self.base + (self.factor * (depth_left as i16))
    }
}
