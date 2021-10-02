#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct TaperedEval(pub i16, pub i16);

impl TaperedEval {
    #[inline]
    pub fn convert(&self, phase: i16) -> i16 {
        let phase = phase as i32;
        ((self.0 as i32 * phase + self.1 as i32 * (TOTAL_PHASE - phase)) / TOTAL_PHASE) as i16
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

//Parameters
pub const PAWN_PHASE: u32 = 0;
pub const KNIGHT_PHASE: u32 = 1;
pub const BISHOP_PHASE: u32 = 1;
pub const ROOK_PHASE: u32 = 2;
pub const QUEEN_PHASE: u32 = 4;
pub const TOTAL_PHASE: i32 =
    (PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2)
        as i32;

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

type T = TaperedEval;
type RankTable = [T; 8];
type SquareTable = [RankTable; 8];
pub const TEMPO: T = TaperedEval(10, 18);
pub const DOUBLED: T = TaperedEval(-1, -27);
pub const ISOLATED: T = TaperedEval(-6, -18);
pub const CHAINED: T = TaperedEval(22, 12);
pub const THREAT: T = TaperedEval(36, 70);
pub const BISHOP_PAIR: T = TaperedEval(17, 101);
pub const PHALANX: T = TaperedEval(7, 15);
pub const PASSED_TABLE: RankTable = [
    TaperedEval(0, 0),
    TaperedEval(-9, 31),
    TaperedEval(-13, 25),
    TaperedEval(-8, 61),
    TaperedEval(25, 91),
    TaperedEval(55, 165),
    TaperedEval(142, 219),
    TaperedEval(0, 0),
];

pub const KNIGHT_ATTACK_CNT: T = TaperedEval(12, 11);
pub const BISHOP_ATTACK_CNT: T = TaperedEval(12, 26);
pub const ROOK_ATTACK_CNT: T = TaperedEval(55, 0);
pub const QUEEN_ATTACK_CNT: T = TaperedEval(8, 63);

pub const PAWN_CNT: T = TaperedEval(88, 186);
pub const KNIGHT_CNT: T = TaperedEval(309, 636);
pub const BISHOP_CNT: T = TaperedEval(324, 675);
pub const ROOK_CNT: T = TaperedEval(435, 1151);
pub const QUEEN_CNT: T = TaperedEval(1090, 1984);
pub const PAWNS: SquareTable = [
    [
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
    ],
    [
        TaperedEval(-42, 5),
        TaperedEval(-32, 0),
        TaperedEval(-42, 4),
        TaperedEval(-39, -2),
        TaperedEval(-40, 14),
        TaperedEval(5, 11),
        TaperedEval(23, -13),
        TaperedEval(-4, -36),
    ],
    [
        TaperedEval(-40, -2),
        TaperedEval(-38, -9),
        TaperedEval(-34, -11),
        TaperedEval(-36, -11),
        TaperedEval(-26, -3),
        TaperedEval(-22, 1),
        TaperedEval(1, -22),
        TaperedEval(-10, -27),
    ],
    [
        TaperedEval(-29, 4),
        TaperedEval(-23, 2),
        TaperedEval(-17, -18),
        TaperedEval(-16, -24),
        TaperedEval(-2, -20),
        TaperedEval(-7, -13),
        TaperedEval(7, -15),
        TaperedEval(-18, -15),
    ],
    [
        TaperedEval(-21, 36),
        TaperedEval(-1, 24),
        TaperedEval(-6, 1),
        TaperedEval(15, -29),
        TaperedEval(34, -20),
        TaperedEval(32, -17),
        TaperedEval(24, 2),
        TaperedEval(-1, -1),
    ],
    [
        TaperedEval(-15, 86),
        TaperedEval(23, 61),
        TaperedEval(41, 27),
        TaperedEval(39, -18),
        TaperedEval(66, -37),
        TaperedEval(131, -5),
        TaperedEval(57, 19),
        TaperedEval(10, 38),
    ],
    [
        TaperedEval(59, 78),
        TaperedEval(50, 69),
        TaperedEval(62, 30),
        TaperedEval(58, -9),
        TaperedEval(38, -20),
        TaperedEval(1, -16),
        TaperedEval(-72, 27),
        TaperedEval(-54, 60),
    ],
    [
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
        TaperedEval(0, 0),
    ],
];
pub const KNIGHTS: SquareTable = [
    [
        TaperedEval(-47, -59),
        TaperedEval(-39, -97),
        TaperedEval(-78, -41),
        TaperedEval(-49, -14),
        TaperedEval(-35, -14),
        TaperedEval(-31, -33),
        TaperedEval(-39, -82),
        TaperedEval(-78, -58),
    ],
    [
        TaperedEval(-59, -52),
        TaperedEval(-57, -6),
        TaperedEval(-21, -4),
        TaperedEval(-17, 15),
        TaperedEval(-23, 14),
        TaperedEval(-16, -12),
        TaperedEval(-29, -15),
        TaperedEval(-29, -42),
    ],
    [
        TaperedEval(-36, -56),
        TaperedEval(-11, 13),
        TaperedEval(-8, 32),
        TaperedEval(11, 54),
        TaperedEval(9, 50),
        TaperedEval(-1, 28),
        TaperedEval(-4, 14),
        TaperedEval(-27, -50),
    ],
    [
        TaperedEval(-18, -10),
        TaperedEval(5, 38),
        TaperedEval(16, 77),
        TaperedEval(15, 85),
        TaperedEval(14, 82),
        TaperedEval(10, 82),
        TaperedEval(43, 46),
        TaperedEval(4, -2),
    ],
    [
        TaperedEval(4, -5),
        TaperedEval(20, 42),
        TaperedEval(52, 67),
        TaperedEval(51, 96),
        TaperedEval(39, 105),
        TaperedEval(61, 83),
        TaperedEval(28, 67),
        TaperedEval(32, 15),
    ],
    [
        TaperedEval(-28, -27),
        TaperedEval(29, 30),
        TaperedEval(68, 57),
        TaperedEval(85, 63),
        TaperedEval(129, 45),
        TaperedEval(126, 70),
        TaperedEval(67, 34),
        TaperedEval(29, -13),
    ],
    [
        TaperedEval(-11, -41),
        TaperedEval(4, -11),
        TaperedEval(53, 15),
        TaperedEval(69, 53),
        TaperedEval(64, 47),
        TaperedEval(88, 6),
        TaperedEval(-1, -7),
        TaperedEval(16, -28),
    ],
    [
        TaperedEval(-104, -88),
        TaperedEval(-6, -20),
        TaperedEval(-21, 6),
        TaperedEval(5, 10),
        TaperedEval(13, 8),
        TaperedEval(-5, 14),
        TaperedEval(-1, -7),
        TaperedEval(-23, -33),
    ],
];
pub const BISHOPS: SquareTable = [
    [
        TaperedEval(-1, -51),
        TaperedEval(-19, -23),
        TaperedEval(-34, -57),
        TaperedEval(-43, -10),
        TaperedEval(-44, -21),
        TaperedEval(-29, -30),
        TaperedEval(-24, -36),
        TaperedEval(-9, -37),
    ],
    [
        TaperedEval(-8, -20),
        TaperedEval(1, -24),
        TaperedEval(3, -7),
        TaperedEval(-13, -1),
        TaperedEval(-8, 5),
        TaperedEval(-3, -21),
        TaperedEval(17, -23),
        TaperedEval(7, -70),
    ],
    [
        TaperedEval(-11, -13),
        TaperedEval(4, 11),
        TaperedEval(4, 28),
        TaperedEval(3, 31),
        TaperedEval(0, 27),
        TaperedEval(2, 22),
        TaperedEval(8, -5),
        TaperedEval(10, -12),
    ],
    [
        TaperedEval(-22, 0),
        TaperedEval(10, 22),
        TaperedEval(5, 42),
        TaperedEval(26, 47),
        TaperedEval(27, 45),
        TaperedEval(6, 40),
        TaperedEval(4, 19),
        TaperedEval(4, -9),
    ],
    [
        TaperedEval(-9, 10),
        TaperedEval(26, 34),
        TaperedEval(32, 37),
        TaperedEval(63, 49),
        TaperedEval(45, 60),
        TaperedEval(49, 40),
        TaperedEval(21, 48),
        TaperedEval(-6, 12),
    ],
    [
        TaperedEval(6, 0),
        TaperedEval(22, 32),
        TaperedEval(54, 33),
        TaperedEval(51, 30),
        TaperedEval(80, 32),
        TaperedEval(97, 46),
        TaperedEval(58, 38),
        TaperedEval(33, 10),
    ],
    [
        TaperedEval(-42, -3),
        TaperedEval(19, 24),
        TaperedEval(-1, 33),
        TaperedEval(-18, 34),
        TaperedEval(4, 37),
        TaperedEval(28, 31),
        TaperedEval(-12, 39),
        TaperedEval(2, -3),
    ],
    [
        TaperedEval(-38, 2),
        TaperedEval(-15, 14),
        TaperedEval(-27, 11),
        TaperedEval(-25, 26),
        TaperedEval(-14, 20),
        TaperedEval(-25, 6),
        TaperedEval(6, 15),
        TaperedEval(-12, 11),
    ],
];
pub const ROOKS: SquareTable = [
    [
        TaperedEval(-41, -17),
        TaperedEval(-28, -11),
        TaperedEval(-20, 0),
        TaperedEval(-17, -3),
        TaperedEval(-24, -11),
        TaperedEval(-32, -5),
        TaperedEval(-11, -9),
        TaperedEval(-28, -35),
    ],
    [
        TaperedEval(-83, -19),
        TaperedEval(-35, -26),
        TaperedEval(-33, -17),
        TaperedEval(-31, -15),
        TaperedEval(-41, -17),
        TaperedEval(-23, -34),
        TaperedEval(-11, -34),
        TaperedEval(-72, -24),
    ],
    [
        TaperedEval(-57, -13),
        TaperedEval(-38, 3),
        TaperedEval(-48, 7),
        TaperedEval(-34, -2),
        TaperedEval(-35, 0),
        TaperedEval(-46, -3),
        TaperedEval(-14, -13),
        TaperedEval(-34, -21),
    ],
    [
        TaperedEval(-37, 11),
        TaperedEval(-38, 36),
        TaperedEval(-35, 37),
        TaperedEval(-20, 31),
        TaperedEval(-26, 25),
        TaperedEval(-27, 18),
        TaperedEval(-15, 18),
        TaperedEval(-17, 0),
    ],
    [
        TaperedEval(-9, 25),
        TaperedEval(5, 36),
        TaperedEval(17, 44),
        TaperedEval(44, 28),
        TaperedEval(30, 29),
        TaperedEval(42, 21),
        TaperedEval(24, 18),
        TaperedEval(21, 16),
    ],
    [
        TaperedEval(5, 38),
        TaperedEval(57, 22),
        TaperedEval(39, 44),
        TaperedEval(65, 28),
        TaperedEval(98, 17),
        TaperedEval(79, 32),
        TaperedEval(92, 16),
        TaperedEval(50, 22),
    ],
    [
        TaperedEval(34, 44),
        TaperedEval(21, 57),
        TaperedEval(55, 55),
        TaperedEval(73, 60),
        TaperedEval(64, 60),
        TaperedEval(85, 26),
        TaperedEval(26, 44),
        TaperedEval(83, 26),
    ],
    [
        TaperedEval(57, 48),
        TaperedEval(52, 49),
        TaperedEval(35, 68),
        TaperedEval(46, 60),
        TaperedEval(40, 67),
        TaperedEval(34, 71),
        TaperedEval(46, 64),
        TaperedEval(75, 60),
    ],
];
pub const QUEENS: SquareTable = [
    [
        TaperedEval(-7, -30),
        TaperedEval(-21, -50),
        TaperedEval(-9, -52),
        TaperedEval(-1, -49),
        TaperedEval(-4, -70),
        TaperedEval(-41, -43),
        TaperedEval(-43, -41),
        TaperedEval(-24, -13),
    ],
    [
        TaperedEval(-10, -38),
        TaperedEval(7, -20),
        TaperedEval(6, -54),
        TaperedEval(3, -14),
        TaperedEval(0, -30),
        TaperedEval(21, -96),
        TaperedEval(31, -111),
        TaperedEval(-3, -26),
    ],
    [
        TaperedEval(-3, -25),
        TaperedEval(5, -5),
        TaperedEval(1, 28),
        TaperedEval(2, -1),
        TaperedEval(4, 3),
        TaperedEval(-4, 21),
        TaperedEval(20, -30),
        TaperedEval(16, -27),
    ],
    [
        TaperedEval(-5, -16),
        TaperedEval(3, 30),
        TaperedEval(3, 30),
        TaperedEval(-7, 87),
        TaperedEval(-7, 68),
        TaperedEval(14, 72),
        TaperedEval(15, 36),
        TaperedEval(29, 49),
    ],
    [
        TaperedEval(-2, -1),
        TaperedEval(-5, 48),
        TaperedEval(0, 36),
        TaperedEval(-4, 92),
        TaperedEval(12, 118),
        TaperedEval(30, 122),
        TaperedEval(32, 111),
        TaperedEval(37, 77),
    ],
    [
        TaperedEval(-11, 13),
        TaperedEval(0, 23),
        TaperedEval(8, 47),
        TaperedEval(27, 62),
        TaperedEval(55, 104),
        TaperedEval(103, 125),
        TaperedEval(116, 95),
        TaperedEval(59, 90),
    ],
    [
        TaperedEval(3, 19),
        TaperedEval(-47, 71),
        TaperedEval(18, 58),
        TaperedEval(12, 88),
        TaperedEval(29, 129),
        TaperedEval(91, 105),
        TaperedEval(12, 69),
        TaperedEval(65, 67),
    ],
    [
        TaperedEval(8, 32),
        TaperedEval(34, 64),
        TaperedEval(40, 66),
        TaperedEval(56, 87),
        TaperedEval(77, 102),
        TaperedEval(89, 106),
        TaperedEval(68, 87),
        TaperedEval(85, 87),
    ],
];
pub const KINGS: SquareTable = [
    [
        TaperedEval(-5, -90),
        TaperedEval(48, -55),
        TaperedEval(-12, -40),
        TaperedEval(-134, -37),
        TaperedEval(-46, -96),
        TaperedEval(-120, -25),
        TaperedEval(20, -63),
        TaperedEval(38, -139),
    ],
    [
        TaperedEval(15, -35),
        TaperedEval(-3, -19),
        TaperedEval(-27, 0),
        TaperedEval(-100, 15),
        TaperedEval(-52, 2),
        TaperedEval(-64, 4),
        TaperedEval(11, -25),
        TaperedEval(29, -65),
    ],
    [
        TaperedEval(-16, -35),
        TaperedEval(-9, -1),
        TaperedEval(-5, 17),
        TaperedEval(-21, 30),
        TaperedEval(-3, 30),
        TaperedEval(-12, 11),
        TaperedEval(15, -15),
        TaperedEval(-33, -38),
    ],
    [
        TaperedEval(-1, -34),
        TaperedEval(18, 21),
        TaperedEval(34, 38),
        TaperedEval(20, 51),
        TaperedEval(35, 47),
        TaperedEval(29, 34),
        TaperedEval(53, 6),
        TaperedEval(-18, -34),
    ],
    [
        TaperedEval(2, -5),
        TaperedEval(19, 46),
        TaperedEval(31, 52),
        TaperedEval(19, 58),
        TaperedEval(23, 53),
        TaperedEval(31, 52),
        TaperedEval(31, 40),
        TaperedEval(-15, -12),
    ],
    [
        TaperedEval(3, 8),
        TaperedEval(21, 59),
        TaperedEval(26, 56),
        TaperedEval(20, 39),
        TaperedEval(17, 30),
        TaperedEval(32, 70),
        TaperedEval(23, 61),
        TaperedEval(2, -1),
    ],
    [
        TaperedEval(-5, -30),
        TaperedEval(11, 31),
        TaperedEval(10, 22),
        TaperedEval(5, 8),
        TaperedEval(7, 9),
        TaperedEval(8, 19),
        TaperedEval(7, 47),
        TaperedEval(-4, -25),
    ],
    [
        TaperedEval(-4, -34),
        TaperedEval(0, -24),
        TaperedEval(0, -12),
        TaperedEval(2, -7),
        TaperedEval(-1, -14),
        TaperedEval(-2, -14),
        TaperedEval(-1, -14),
        TaperedEval(-3, -28),
    ],
];

pub const PAWN_TABLE: SquareTable = add_piece(PAWNS, PAWN_CNT);
pub const KNIGHT_TABLE: SquareTable = add_piece(KNIGHTS, KNIGHT_CNT);
pub const BISHOP_TABLE: SquareTable = add_piece(BISHOPS, BISHOP_CNT);
pub const ROOK_TABLE: SquareTable = add_piece(ROOKS, ROOK_CNT);
pub const QUEEN_TABLE: SquareTable = add_piece(QUEENS, QUEEN_CNT);
pub const KING_TABLE: SquareTable = KINGS;
