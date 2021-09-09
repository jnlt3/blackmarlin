use crate::bm::bm_eval::eval::Evaluation;

#[derive(Debug, Clone)]
pub struct Window {
    start: i32,
    factor: i32,
    divisor: i32,
    add: i32,

    center: Evaluation,
    upper_window: i32,
    lower_window: i32,
}

impl Window {
    pub const fn new(start: i32, factor: i32, divisor: i32, add: i32) -> Self {
        Self {
            start,
            factor,
            divisor,
            add,
            center: Evaluation::new(0),
            upper_window: start,
            lower_window: start,
        }
    }

    pub fn set(&mut self, eval: Evaluation) {
        self.center = eval;
    }

    pub fn get(&self) -> (Evaluation, Evaluation) {
        (
            self.center - self.lower_window,
            self.center + self.upper_window,
        )
    }

    pub fn fail_low(&mut self) {
        self.lower_window = (self.lower_window * self.factor) / self.divisor + self.add;
    }

    pub fn fail_high(&mut self) {
        self.upper_window = (self.upper_window * self.factor) / self.divisor + self.add;
    }
}
