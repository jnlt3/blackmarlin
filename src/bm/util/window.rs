use crate::bm::util::eval::Evaluation;

#[derive(Debug, Clone)]
pub struct Window {
    start: i16,
    factor: i16,
    divisor: i16,
    add: i16,

    center: Evaluation,
    alpha: Evaluation,
    beta: Evaluation,
    window: i16,
}

impl Window {
    pub const fn new(start: i16, factor: i16, divisor: i16, add: i16) -> Self {
        Self {
            start,
            factor,
            divisor,
            add,
            center: Evaluation::new(0),
            alpha: Evaluation::new(start),
            beta: Evaluation::new(start),
            window: start,
        }
    }

    pub fn reset(&mut self) {
        self.window = self.start;
        self.set_bounds();
    }

    pub fn set(&mut self, eval: Evaluation) {
        self.center = eval;
    }

    pub fn get(&self) -> (Evaluation, Evaluation) {
        (self.alpha, self.beta)
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
        self.alpha = self.center - self.window;
        self.beta = self.center + self.window;
    }
}
