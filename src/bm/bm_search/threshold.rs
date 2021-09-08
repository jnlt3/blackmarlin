#[derive(Debug, Clone)]
pub struct Threshold {
    base: i32,
    factor: i32,
}

impl Threshold {
    pub const fn new(base: i32, factor: i32) -> Self {
        Self { base, factor }
    }

    #[inline]
    pub fn threshold(&self, depth_left: u32) -> i32 {
        self.base + (self.factor * (depth_left as i32))
    }
}
