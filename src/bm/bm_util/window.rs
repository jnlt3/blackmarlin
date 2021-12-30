use crate::bm::bm_eval::eval::Evaluation;

#[derive(Debug, Clone)]
pub struct Window {
    start: i16,
    factor: i16,
    divisor: i16,
    add: i16,

    center: i16,
    alpha: i16,
    beta: i16,
    window: i16,
}

impl Window {
    pub const fn new(start: i16, factor: i16, divisor: i16, add: i16) -> Self {
        Self {
            start,
            factor,
            divisor,
            add,
            center: 0,
            alpha: start,
            beta: start,
            window: start,
        }
    }

    pub fn reset(&mut self) {
        self.window = self.start;
        self.set_bounds();
    }

    pub fn set(&mut self, eval: Evaluation) {
        self.center = eval.raw();
    }

    pub fn get(&self) -> (Evaluation, Evaluation) {
        (Evaluation::new(self.alpha), Evaluation::new(self.beta))
    }

    pub fn fail_low(&mut self) {
        self.beta = (self.alpha + self.beta) / 2;
        self.alpha = self.center - self.window;
        self.expand();
    }

    pub fn fail_high(&mut self) {
        self.beta = self.center + self.window;
        self.expand();
    }

    fn expand(&mut self) {
        self.window += self.window * self.factor / self.divisor + self.add;
    }

    fn set_bounds(&mut self) {
        self.alpha = self.center.saturating_sub(self.window);
        self.beta = self.center.saturating_add(self.window);
    }
}
