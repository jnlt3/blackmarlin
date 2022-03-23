#[derive(Debug, Copy, Clone)]
pub struct Adapt {
    pub singular: bool,
}

impl Default for Adapt {
    fn default() -> Self {
        Self { singular: true }
    }
}
