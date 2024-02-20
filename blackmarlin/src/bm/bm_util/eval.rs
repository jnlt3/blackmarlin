const CHECKMATE: i16 = 128;
const CHECKMATE_EVAL: i16 = i16::MAX - 1024;
const MAX_EVAL: i16 = CHECKMATE_EVAL - CHECKMATE;

pub enum Depth {
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Evaluation {
    score: i16,
}

impl Evaluation {
    pub const fn new(score: i16) -> Self {
        Self { score }
    }

    pub fn new_checkmate(mate_in: i16) -> Self {
        Self {
            score: if mate_in < 0 {
                -CHECKMATE_EVAL - mate_in
            } else {
                CHECKMATE_EVAL - mate_in
            },
        }
    }

    pub const fn is_mate(&self) -> bool {
        self.score.saturating_abs() > MAX_EVAL
    }

    pub const fn mate_in(&self) -> Option<i16> {
        if self.is_mate() {
            Some(self.score.signum() * ((CHECKMATE_EVAL - self.score.abs()) / 2))
        } else {
            None
        }
    }

    pub const fn raw(&self) -> i16 {
        self.score
    }

    pub const fn min() -> Self {
        Self {
            score: -CHECKMATE_EVAL,
        }
    }

    pub const fn max() -> Self {
        Self {
            score: CHECKMATE_EVAL,
        }
    }
}

impl std::ops::Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { score: -self.score }
    }
}

impl std::ops::Shl<Depth> for Evaluation {
    type Output = Self;

    fn shl(self, _: Depth) -> Self::Output {
        let sign = if self.is_mate() {
            self.score.signum()
        } else {
            0
        };
        Self {
            score: self.score.saturating_neg().saturating_add(sign),
        }
    }
}

impl std::ops::Shr<Depth> for Evaluation {
    type Output = Self;

    fn shr(self, _: Depth) -> Self::Output {
        let sign = if self.is_mate() {
            self.score.signum()
        } else {
            0
        };
        Self {
            score: self.score.saturating_neg().saturating_add(sign),
        }
    }
}

macro_rules! impl_i16_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {
        $(
            impl std::ops::$trait<i16> for Evaluation {
                type Output = Self;

                fn $fn(self, rhs: i16) -> Self::Output {
                    Evaluation {
                        score: self.score.$op(rhs)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_eval_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {
        $(
            impl std::ops::$trait<Evaluation> for Evaluation {
                type Output = Self;

                fn $fn(self, rhs: Evaluation) -> Self::Output {
                    Evaluation {
                        score: self.score.$op(rhs.score)
                    }
                }
            }
        )*
    };
}

impl_i16_ops! {
    Add, add, add;
    Sub, sub, sub;
    Mul, mul, mul;
    Div, div, div;
}

impl_eval_ops! {
    Add, add, add;
    Sub, sub, sub;
    Mul, mul, mul;
    Div, div, div;
}

#[test]
fn mate_comparisons() {
    let w_checkmate_in_2 = Evaluation::new_checkmate(2);
    let w_checkmate_in_1 = Evaluation::new_checkmate(1);
    let b_checkmate_in_1 = Evaluation::new_checkmate(-1);
    let b_checkmate_in_2 = Evaluation::new_checkmate(-2);

    assert_eq!(w_checkmate_in_2.mate_in().unwrap(), 2);
    assert_eq!(w_checkmate_in_1.mate_in().unwrap(), 1);
    assert_eq!(b_checkmate_in_1.mate_in().unwrap(), -1);
    assert_eq!(b_checkmate_in_2.mate_in().unwrap(), -2);

    assert!(w_checkmate_in_2 >= w_checkmate_in_2);
    assert!(w_checkmate_in_1 >= w_checkmate_in_1);
    assert!(b_checkmate_in_1 >= b_checkmate_in_1);
    assert!(b_checkmate_in_2 >= b_checkmate_in_2);

    assert!(w_checkmate_in_2 <= w_checkmate_in_2);
    assert!(w_checkmate_in_1 <= w_checkmate_in_1);
    assert!(b_checkmate_in_1 <= b_checkmate_in_1);
    assert!(b_checkmate_in_2 <= b_checkmate_in_2);

    assert!(w_checkmate_in_2 == w_checkmate_in_2);
    assert!(w_checkmate_in_1 == w_checkmate_in_1);
    assert!(b_checkmate_in_1 == b_checkmate_in_1);
    assert!(b_checkmate_in_2 == b_checkmate_in_2);

    assert!(w_checkmate_in_1 > w_checkmate_in_2);
    assert!(w_checkmate_in_2 > b_checkmate_in_1);
    assert!(w_checkmate_in_2 > b_checkmate_in_2);
    assert!(w_checkmate_in_1 > b_checkmate_in_1);
    assert!(w_checkmate_in_1 > b_checkmate_in_2);
    assert!(b_checkmate_in_2 > b_checkmate_in_1);

    assert!(w_checkmate_in_1 >= w_checkmate_in_2);
    assert!(w_checkmate_in_2 >= b_checkmate_in_1);
    assert!(w_checkmate_in_2 >= b_checkmate_in_2);
    assert!(w_checkmate_in_1 >= b_checkmate_in_1);
    assert!(w_checkmate_in_1 >= b_checkmate_in_2);
    assert!(b_checkmate_in_2 >= b_checkmate_in_1);

    assert!(w_checkmate_in_2 < w_checkmate_in_1);
    assert!(b_checkmate_in_1 < w_checkmate_in_2);
    assert!(b_checkmate_in_2 < w_checkmate_in_2);
    assert!(b_checkmate_in_1 < w_checkmate_in_1);
    assert!(b_checkmate_in_2 < w_checkmate_in_1);
    assert!(b_checkmate_in_1 < b_checkmate_in_2);

    assert!(w_checkmate_in_2 <= w_checkmate_in_1);
    assert!(b_checkmate_in_1 <= w_checkmate_in_2);
    assert!(b_checkmate_in_2 <= w_checkmate_in_2);
    assert!(b_checkmate_in_1 <= w_checkmate_in_1);
    assert!(b_checkmate_in_2 <= w_checkmate_in_1);
    assert!(b_checkmate_in_1 <= b_checkmate_in_2);
}
