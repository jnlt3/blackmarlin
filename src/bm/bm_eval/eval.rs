const CHECKMATE: i32 = 64;
const CHECKMATE_EVAL: i32 = i32::MAX;
const MAX_EVAL: i32 = CHECKMATE_EVAL - CHECKMATE;

pub enum Depth {
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Evaluation {
    score: i32,
}

impl Evaluation {
    #[inline]
    pub const fn new(score: i32) -> Self {
        Self { score }
    }

    #[inline]
    pub fn new_checkmate(mate_in: i32) -> Self {
        Self {
            score: if mate_in < 0 {
                -CHECKMATE_EVAL - mate_in - 1
            } else {
                CHECKMATE_EVAL - mate_in + 1
            },
        }
    }

    #[inline]
    pub const fn is_mate(&self) -> bool {
        self.score.saturating_abs() > MAX_EVAL
    }

    #[inline]
    pub const fn mate_in(&self) -> Option<i32> {
        if self.is_mate() {
            Some(self.score.signum() * (MAX_EVAL - self.score.abs()))
        } else {
            None
        }
    }

    #[inline]
    pub const fn raw(&self) -> i32 {
        self.score
    }

    #[inline]
    pub const fn min() -> Self {
        Self {
            score: -CHECKMATE_EVAL,
        }
    }

    #[inline]
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
            score: self.score.saturating_add(sign).saturating_neg(),
        }
    }
}

macro_rules! impl_i32_ops {
    ($($trait:ident, $fn:ident, $op:ident;)*) => {
        $(
            impl std::ops::$trait<i32> for Evaluation {
                type Output = Self;

                fn $fn(self, rhs: i32) -> Self::Output {
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

impl_i32_ops! {
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
