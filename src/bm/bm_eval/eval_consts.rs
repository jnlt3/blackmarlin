#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct TaperedEval(pub i16, pub i16);

impl TaperedEval {
    #[inline]
    pub fn convert(&self, phase: i16) -> i16 {
        let phase = phase as i32;
        ((self.0 as i32 * phase + self.1 as i32 * (TOTAL_PHASE as i32 - phase))
            / TOTAL_PHASE as i32) as i16
    }
}

macro_rules! impl_tapered_eval_op {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait for TaperedEval {
            type Output = TaperedEval;

            fn $op(self, rhs: Self) -> Self::Output {
                TaperedEval(self.0.$op(rhs.0), self.1.$op(rhs.1))
            }
        }
    };
}

macro_rules! impl_tapered_eval_op_assign {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait for TaperedEval {
            fn $op(&mut self, rhs: Self) {
                self.0.$op(rhs.0);
                self.1.$op(rhs.1);
            }
        }
    };
}

macro_rules! impl_tapered_eval_i16_op {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait<i16> for TaperedEval {
            type Output = TaperedEval;

            fn $op(self, rhs: i16) -> Self::Output {
                TaperedEval(self.0.$op(rhs), self.1.$op(rhs))
            }
        }
        impl std::ops::$trait<TaperedEval> for i16 {
            type Output = TaperedEval;

            fn $op(self, rhs: TaperedEval) -> Self::Output {
                TaperedEval(self.$op(rhs.0), self.$op(rhs.1))
            }
        }
    };
}

macro_rules! impl_tapered_eval_i16_op_assign {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait<i16> for TaperedEval {
            fn $op(&mut self, rhs: i16) {
                self.0.$op(rhs);
                self.1.$op(rhs);
            }
        }
    };
}

impl_tapered_eval_op!(Add, add);
impl_tapered_eval_op!(Sub, sub);
impl_tapered_eval_op_assign!(AddAssign, add_assign);
impl_tapered_eval_op_assign!(SubAssign, sub_assign);
impl_tapered_eval_i16_op!(Add, add);
impl_tapered_eval_i16_op!(Sub, sub);
impl_tapered_eval_i16_op!(Mul, mul);
impl_tapered_eval_i16_op!(Div, div);
impl_tapered_eval_i16_op_assign!(AddAssign, add_assign);
impl_tapered_eval_i16_op_assign!(SubAssign, sub_assign);
impl_tapered_eval_i16_op_assign!(MulAssign, mul_assign);

pub const PAWN_PHASE: u32 = 0;
pub const KNIGHT_PHASE: u32 = 1;
pub const BISHOP_PHASE: u32 = 1;
pub const ROOK_PHASE: u32 = 2;
pub const QUEEN_PHASE: u32 = 4;
pub const TOTAL_PHASE: u32 =
    PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

const fn add_piece(
    table: [[TaperedEval; 8]; 8],
    piece_value: TaperedEval,
) -> [[TaperedEval; 8]; 8] {
    let mut combined = [[TaperedEval(0, 0); 8]; 8];
    let mut x = 0;
    let mut y = 0;
    while y < 8 {
        combined[x][y] = TaperedEval(table[x][y].0 + piece_value.0, table[x][y].1 + piece_value.1);
        if x < 7 {
            x += 1;
        } else {
            x = 0;
            y += 1;
        }
    }
    combined
}

macro_rules! E {
    ($x:expr, $y:expr) => {
        TaperedEval($x, $y)
    };
}

type T = TaperedEval;
type RankTable = [T; 8];
type SquareTable = [RankTable; 8];
type IndexTable<const SIZE: usize> = [TaperedEval; SIZE];
pub const TEMPO: T = E!(12, 16);
pub const DOUBLED: T = E!(-8, -19);
pub const ISOLATED: T = E!(-9, -12);
pub const CHAINED: T = E!(16, 13);
pub const THREAT: T = E!(34, 58);
pub const BISHOP_PAIR: T = E!(21, 88);
pub const PHALANX: T = E!(5, 11);
pub const PASSED_TABLE: RankTable = [
    E!(0, 0),
    E!(-7, 18),
    E!(-14, 21),
    E!(-7, 49),
    E!(20, 74),
    E!(39, 141),
    E!(91, 187),
    E!(0, 0),
];
pub const KNIGHT_MOBILITY: IndexTable<9> = [
    E!(-12, -83),
    E!(6, -5),
    E!(15, 44),
    E!(21, 62),
    E!(32, 78),
    E!(40, 95),
    E!(43, 95),
    E!(51, 87),
    E!(63, 62),
];
pub const BISHOP_MOBILITY: IndexTable<14> = [
    E!(-21, -104),
    E!(-6, -55),
    E!(0, -6),
    E!(0, 14),
    E!(10, 37),
    E!(18, 51),
    E!(19, 66),
    E!(16, 71),
    E!(19, 78),
    E!(25, 77),
    E!(28, 73),
    E!(47, 59),
    E!(57, 82),
    E!(79, 42),
];
pub const ROOK_MOBILITY: IndexTable<15> = [
    E!(-77, -79),
    E!(-8, -23),
    E!(4, 22),
    E!(5, 39),
    E!(5, 55),
    E!(8, 63),
    E!(18, 74),
    E!(22, 81),
    E!(33, 82),
    E!(39, 88),
    E!(43, 95),
    E!(48, 99),
    E!(55, 103),
    E!(76, 87),
    E!(135, 58),
];
pub const QUEEN_MOBILITY: IndexTable<28> = [
    E!(-1, 0),
    E!(-16, -7),
    E!(-16, -22),
    E!(7, -48),
    E!(15, -50),
    E!(21, -6),
    E!(22, 16),
    E!(22, 41),
    E!(29, 59),
    E!(33, 74),
    E!(38, 87),
    E!(37, 87),
    E!(38, 93),
    E!(41, 98),
    E!(42, 102),
    E!(42, 106),
    E!(47, 108),
    E!(47, 103),
    E!(47, 111),
    E!(50, 108),
    E!(58, 99),
    E!(76, 84),
    E!(60, 88),
    E!(70, 84),
    E!(49, 69),
    E!(35, 54),
    E!(14, 25),
    E!(7, 13),
];
pub const ATTACKERS: IndexTable<16> = [
    E!(-82, 12),
    E!(-55, 9),
    E!(-55, 9),
    E!(-55, 9),
    E!(-79, 4),
    E!(-65, 0),
    E!(24, -16),
    E!(24, -16),
    E!(-69, 7),
    E!(-23, -1),
    E!(109, -36),
    E!(32, -7),
    E!(-45, 21),
    E!(38, -15),
    E!(137, -30),
    E!(78, 52),
];
pub const KNIGHT_ATTACK_CNT: T = E!(6, 9);
pub const BISHOP_ATTACK_CNT: T = E!(4, 18);
pub const ROOK_ATTACK_CNT: T = E!(26, -1);
pub const QUEEN_ATTACK_CNT: T = E!(2, 38);
pub const PAWN_CNT: T = E!(81, 150);
pub const KNIGHT_CNT: T = E!(261, 435);
pub const BISHOP_CNT: T = E!(289, 486);
pub const ROOK_CNT: T = E!(405, 843);
pub const QUEEN_CNT: T = E!(915, 1577);
pub const PAWNS: SquareTable = [
    [
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
    ],
    [
        E!(-22, 3),
        E!(-26, 1),
        E!(-24, 0),
        E!(-11, -1),
        E!(-19, 11),
        E!(15, 6),
        E!(29, -13),
        E!(-1, -28),
    ],
    [
        E!(-25, -6),
        E!(-27, -8),
        E!(-19, -8),
        E!(-20, -10),
        E!(-4, -2),
        E!(-10, -4),
        E!(7, -19),
        E!(-6, -21),
    ],
    [
        E!(-20, 5),
        E!(-23, 4),
        E!(-14, -17),
        E!(-6, -19),
        E!(9, -17),
        E!(2, -11),
        E!(-1, -11),
        E!(-8, -15),
    ],
    [
        E!(-17, 33),
        E!(-14, 20),
        E!(-10, 6),
        E!(5, -27),
        E!(17, -18),
        E!(26, -12),
        E!(7, 4),
        E!(-1, 0),
    ],
    [
        E!(-17, 68),
        E!(14, 51),
        E!(18, 25),
        E!(22, -14),
        E!(35, -30),
        E!(91, -3),
        E!(39, 12),
        E!(0, 30),
    ],
    [
        E!(71, 59),
        E!(53, 54),
        E!(62, 21),
        E!(61, -18),
        E!(49, -27),
        E!(-6, -8),
        E!(-114, 45),
        E!(-84, 61),
    ],
    [
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
        E!(0, 0),
    ],
];
pub const KNIGHTS: SquareTable = [
    [
        E!(-49, -41),
        E!(-4, -33),
        E!(-37, -17),
        E!(-2, 6),
        E!(4, 2),
        E!(-9, -5),
        E!(-11, -17),
        E!(-62, -39),
    ],
    [
        E!(-25, -18),
        E!(-19, 4),
        E!(-11, -12),
        E!(1, 1),
        E!(7, 7),
        E!(3, -13),
        E!(-7, 8),
        E!(-3, -3),
    ],
    [
        E!(-13, -36),
        E!(-1, -7),
        E!(-2, 5),
        E!(15, 30),
        E!(12, 23),
        E!(9, -2),
        E!(8, -2),
        E!(1, -19),
    ],
    [
        E!(0, 1),
        E!(13, 14),
        E!(18, 45),
        E!(14, 51),
        E!(16, 50),
        E!(17, 51),
        E!(42, 25),
        E!(12, 19),
    ],
    [
        E!(13, 0),
        E!(25, 18),
        E!(35, 46),
        E!(40, 59),
        E!(30, 64),
        E!(46, 55),
        E!(27, 38),
        E!(41, 19),
    ],
    [
        E!(-27, -13),
        E!(23, 8),
        E!(30, 37),
        E!(44, 38),
        E!(62, 35),
        E!(54, 52),
        E!(7, 18),
        E!(-14, 3),
    ],
    [
        E!(-4, -14),
        E!(2, -7),
        E!(38, -8),
        E!(61, 20),
        E!(46, 20),
        E!(43, -21),
        E!(-26, 1),
        E!(12, -9),
    ],
    [
        E!(-133, -76),
        E!(-14, -17),
        E!(-54, 15),
        E!(-2, 5),
        E!(-5, 3),
        E!(-32, 14),
        E!(-2, -1),
        E!(-42, -46),
    ],
];
pub const BISHOPS: SquareTable = [
    [
        E!(21, -14),
        E!(19, -7),
        E!(8, -15),
        E!(-8, -1),
        E!(-6, -3),
        E!(8, 2),
        E!(16, -14),
        E!(29, -11),
    ],
    [
        E!(16, -4),
        E!(21, -19),
        E!(22, -11),
        E!(7, 4),
        E!(15, -1),
        E!(22, -11),
        E!(36, -10),
        E!(29, -28),
    ],
    [
        E!(10, -4),
        E!(18, 5),
        E!(14, 8),
        E!(16, 9),
        E!(21, 13),
        E!(24, 8),
        E!(24, -3),
        E!(19, -1),
    ],
    [
        E!(-4, -4),
        E!(14, 1),
        E!(17, 17),
        E!(36, 19),
        E!(28, 16),
        E!(5, 18),
        E!(6, 7),
        E!(11, -14),
    ],
    [
        E!(0, -3),
        E!(27, 12),
        E!(21, 13),
        E!(58, 22),
        E!(39, 28),
        E!(37, 11),
        E!(21, 23),
        E!(-7, 14),
    ],
    [
        E!(13, 2),
        E!(30, 16),
        E!(34, 15),
        E!(26, 10),
        E!(46, 12),
        E!(37, 31),
        E!(12, 26),
        E!(-19, 19),
    ],
    [
        E!(-43, 14),
        E!(13, 10),
        E!(-7, 13),
        E!(-34, 20),
        E!(-39, 23),
        E!(-26, 22),
        E!(-48, 30),
        E!(-48, 20),
    ],
    [
        E!(-43, 18),
        E!(-33, 18),
        E!(-59, 21),
        E!(-67, 26),
        E!(-53, 14),
        E!(-78, 6),
        E!(-4, 13),
        E!(-27, 15),
    ],
];
pub const ROOKS: SquareTable = [
    [
        E!(-18, -20),
        E!(-4, -11),
        E!(1, -14),
        E!(9, -18),
        E!(2, -20),
        E!(1, -7),
        E!(4, -13),
        E!(-10, -34),
    ],
    [
        E!(-49, -17),
        E!(-19, -24),
        E!(-12, -12),
        E!(-6, -15),
        E!(-12, -17),
        E!(-1, -27),
        E!(5, -28),
        E!(-44, -18),
    ],
    [
        E!(-35, -6),
        E!(-21, -1),
        E!(-28, 4),
        E!(-13, -5),
        E!(-21, -3),
        E!(-16, -2),
        E!(5, -3),
        E!(-17, -13),
    ],
    [
        E!(-23, 8),
        E!(-23, 26),
        E!(-20, 26),
        E!(-13, 21),
        E!(-13, 15),
        E!(-11, 20),
        E!(7, 16),
        E!(-12, 8),
    ],
    [
        E!(-2, 22),
        E!(13, 23),
        E!(12, 28),
        E!(43, 18),
        E!(20, 21),
        E!(31, 17),
        E!(22, 12),
        E!(17, 18),
    ],
    [
        E!(-7, 35),
        E!(47, 17),
        E!(24, 33),
        E!(45, 15),
        E!(43, 22),
        E!(31, 31),
        E!(69, 6),
        E!(30, 21),
    ],
    [
        E!(9, 41),
        E!(2, 47),
        E!(22, 47),
        E!(41, 45),
        E!(2, 62),
        E!(43, 23),
        E!(1, 36),
        E!(52, 22),
    ],
    [
        E!(36, 33),
        E!(31, 42),
        E!(9, 55),
        E!(9, 48),
        E!(-7, 53),
        E!(12, 54),
        E!(44, 43),
        E!(62, 39),
    ],
];
pub const QUEENS: SquareTable = [
    [
        E!(14, -56),
        E!(19, -74),
        E!(22, -74),
        E!(35, -44),
        E!(33, -76),
        E!(-3, -57),
        E!(-7, -52),
        E!(8, -23),
    ],
    [
        E!(21, -56),
        E!(20, -35),
        E!(25, -66),
        E!(16, -31),
        E!(19, -41),
        E!(40, -97),
        E!(47, -113),
        E!(24, -41),
    ],
    [
        E!(12, -48),
        E!(18, -23),
        E!(16, 11),
        E!(8, -15),
        E!(13, -4),
        E!(10, 18),
        E!(32, -20),
        E!(32, -28),
    ],
    [
        E!(11, -31),
        E!(16, 10),
        E!(6, 10),
        E!(-6, 66),
        E!(-3, 60),
        E!(13, 59),
        E!(16, 40),
        E!(28, 61),
    ],
    [
        E!(4, -15),
        E!(-2, 34),
        E!(-9, 28),
        E!(-4, 81),
        E!(0, 102),
        E!(11, 115),
        E!(24, 109),
        E!(35, 80),
    ],
    [
        E!(-9, 10),
        E!(-4, 16),
        E!(-15, 46),
        E!(4, 65),
        E!(9, 106),
        E!(42, 124),
        E!(59, 102),
        E!(24, 114),
    ],
    [
        E!(-3, 23),
        E!(-56, 71),
        E!(-12, 61),
        E!(-25, 94),
        E!(-45, 154),
        E!(31, 99),
        E!(-33, 103),
        E!(33, 80),
    ],
    [
        E!(0, 29),
        E!(15, 55),
        E!(16, 61),
        E!(27, 69),
        E!(38, 82),
        E!(73, 86),
        E!(58, 88),
        E!(77, 78),
    ],
];
pub const KINGS: SquareTable = [
    [
        E!(-33, -73),
        E!(19, -43),
        E!(-27, -20),
        E!(-119, -24),
        E!(-49, -55),
        E!(-103, -14),
        E!(-10, -48),
        E!(-7, -109),
    ],
    [
        E!(-18, -19),
        E!(-28, -5),
        E!(-42, 5),
        E!(-104, 20),
        E!(-62, 10),
        E!(-69, 13),
        E!(-22, -15),
        E!(-10, -39),
    ],
    [
        E!(-30, -20),
        E!(-16, 2),
        E!(-2, 13),
        E!(-21, 32),
        E!(3, 24),
        E!(-10, 15),
        E!(-8, -7),
        E!(-50, -17),
    ],
    [
        E!(1, -23),
        E!(30, 18),
        E!(59, 29),
        E!(37, 48),
        E!(54, 36),
        E!(38, 29),
        E!(49, 4),
        E!(-28, -21),
    ],
    [
        E!(2, -2),
        E!(33, 37),
        E!(53, 41),
        E!(31, 46),
        E!(48, 44),
        E!(57, 40),
        E!(51, 29),
        E!(-26, -3),
    ],
    [
        E!(3, 3),
        E!(38, 47),
        E!(47, 40),
        E!(35, 32),
        E!(36, 23),
        E!(55, 51),
        E!(35, 49),
        E!(1, 1),
    ],
    [
        E!(-7, -44),
        E!(19, 32),
        E!(23, 26),
        E!(11, 7),
        E!(15, 10),
        E!(17, 17),
        E!(11, 42),
        E!(-4, -27),
    ],
    [
        E!(-9, -74),
        E!(1, -48),
        E!(3, -24),
        E!(3, -14),
        E!(2, -23),
        E!(-1, -18),
        E!(-1, -26),
        E!(-5, -61),
    ],
];

pub const PAWN_TABLE: SquareTable = add_piece(PAWNS, PAWN_CNT);
pub const KNIGHT_TABLE: SquareTable = add_piece(KNIGHTS, KNIGHT_CNT);
pub const BISHOP_TABLE: SquareTable = add_piece(BISHOPS, BISHOP_CNT);
pub const ROOK_TABLE: SquareTable = add_piece(ROOKS, ROOK_CNT);
pub const QUEEN_TABLE: SquareTable = add_piece(QUEENS, QUEEN_CNT);
pub const KING_TABLE: SquareTable = KINGS;
