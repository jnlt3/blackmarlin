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
type IndexTable<const SIZE: usize> = [TaperedEval; SIZE];
pub const TEMPO: T = TaperedEval(12, 16);
pub const DOUBLED: T = TaperedEval(-8, -19);
pub const ISOLATED: T = TaperedEval(-9, -12);
pub const CHAINED: T = TaperedEval(16, 13);
pub const THREAT: T = TaperedEval(34, 58);
pub const BISHOP_PAIR: T = TaperedEval(21, 88);
pub const PHALANX: T = TaperedEval(5, 11);
pub const PASSED_TABLE: RankTable = [
    TaperedEval(0, 0),
    TaperedEval(-7, 18),
    TaperedEval(-14, 21),
    TaperedEval(-7, 49),
    TaperedEval(20, 74),
    TaperedEval(39, 141),
    TaperedEval(91, 187),
    TaperedEval(0, 0),
];
pub const KNIGHT_MOBILITY: IndexTable<9> = [
    TaperedEval(-12, -83),
    TaperedEval(6, -5),
    TaperedEval(15, 44),
    TaperedEval(21, 62),
    TaperedEval(32, 78),
    TaperedEval(40, 95),
    TaperedEval(43, 95),
    TaperedEval(51, 87),
    TaperedEval(63, 62),
];
pub const BISHOP_MOBILITY: IndexTable<14> = [
    TaperedEval(-21, -104),
    TaperedEval(-6, -55),
    TaperedEval(0, -6),
    TaperedEval(0, 14),
    TaperedEval(10, 37),
    TaperedEval(18, 51),
    TaperedEval(19, 66),
    TaperedEval(16, 71),
    TaperedEval(19, 78),
    TaperedEval(25, 77),
    TaperedEval(28, 73),
    TaperedEval(47, 59),
    TaperedEval(57, 82),
    TaperedEval(79, 42),
];
pub const ROOK_MOBILITY: IndexTable<15> = [
    TaperedEval(-77, -79),
    TaperedEval(-8, -23),
    TaperedEval(4, 22),
    TaperedEval(5, 39),
    TaperedEval(5, 55),
    TaperedEval(8, 63),
    TaperedEval(18, 74),
    TaperedEval(22, 81),
    TaperedEval(33, 82),
    TaperedEval(39, 88),
    TaperedEval(43, 95),
    TaperedEval(48, 99),
    TaperedEval(55, 103),
    TaperedEval(76, 87),
    TaperedEval(135, 58),
];
pub const QUEEN_MOBILITY: IndexTable<28> = [
    TaperedEval(-1, 0),
    TaperedEval(-16, -7),
    TaperedEval(-16, -22),
    TaperedEval(7, -48),
    TaperedEval(15, -50),
    TaperedEval(21, -6),
    TaperedEval(22, 16),
    TaperedEval(22, 41),
    TaperedEval(29, 59),
    TaperedEval(33, 74),
    TaperedEval(38, 87),
    TaperedEval(37, 87),
    TaperedEval(38, 93),
    TaperedEval(41, 98),
    TaperedEval(42, 102),
    TaperedEval(42, 106),
    TaperedEval(47, 108),
    TaperedEval(47, 103),
    TaperedEval(47, 111),
    TaperedEval(50, 108),
    TaperedEval(58, 99),
    TaperedEval(76, 84),
    TaperedEval(60, 88),
    TaperedEval(70, 84),
    TaperedEval(49, 69),
    TaperedEval(35, 54),
    TaperedEval(14, 25),
    TaperedEval(7, 13),
];
pub const ATTACKERS: IndexTable<16> = [
    TaperedEval(-82, 12),
    TaperedEval(-55, 9),
    TaperedEval(-55, 9),
    TaperedEval(-55, 9),
    TaperedEval(-79, 4),
    TaperedEval(-65, 0),
    TaperedEval(24, -16),
    TaperedEval(24, -16),
    TaperedEval(-69, 7),
    TaperedEval(-23, -1),
    TaperedEval(109, -36),
    TaperedEval(32, -7),
    TaperedEval(-45, 21),
    TaperedEval(38, -15),
    TaperedEval(137, -30),
    TaperedEval(78, 52),
];
pub const KNIGHT_ATTACK_CNT: T = TaperedEval(6, 9);
pub const BISHOP_ATTACK_CNT: T = TaperedEval(4, 18);
pub const ROOK_ATTACK_CNT: T = TaperedEval(26, -1);
pub const QUEEN_ATTACK_CNT: T = TaperedEval(2, 38);
pub const PAWN_CNT: T = TaperedEval(81, 150);
pub const KNIGHT_CNT: T = TaperedEval(261, 435);
pub const BISHOP_CNT: T = TaperedEval(289, 486);
pub const ROOK_CNT: T = TaperedEval(405, 843);
pub const QUEEN_CNT: T = TaperedEval(915, 1577);
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
        TaperedEval(-22, 3),
        TaperedEval(-26, 1),
        TaperedEval(-24, 0),
        TaperedEval(-11, -1),
        TaperedEval(-19, 11),
        TaperedEval(15, 6),
        TaperedEval(29, -13),
        TaperedEval(-1, -28),
    ],
    [
        TaperedEval(-25, -6),
        TaperedEval(-27, -8),
        TaperedEval(-19, -8),
        TaperedEval(-20, -10),
        TaperedEval(-4, -2),
        TaperedEval(-10, -4),
        TaperedEval(7, -19),
        TaperedEval(-6, -21),
    ],
    [
        TaperedEval(-20, 5),
        TaperedEval(-23, 4),
        TaperedEval(-14, -17),
        TaperedEval(-6, -19),
        TaperedEval(9, -17),
        TaperedEval(2, -11),
        TaperedEval(-1, -11),
        TaperedEval(-8, -15),
    ],
    [
        TaperedEval(-17, 33),
        TaperedEval(-14, 20),
        TaperedEval(-10, 6),
        TaperedEval(5, -27),
        TaperedEval(17, -18),
        TaperedEval(26, -12),
        TaperedEval(7, 4),
        TaperedEval(-1, 0),
    ],
    [
        TaperedEval(-17, 68),
        TaperedEval(14, 51),
        TaperedEval(18, 25),
        TaperedEval(22, -14),
        TaperedEval(35, -30),
        TaperedEval(91, -3),
        TaperedEval(39, 12),
        TaperedEval(0, 30),
    ],
    [
        TaperedEval(71, 59),
        TaperedEval(53, 54),
        TaperedEval(62, 21),
        TaperedEval(61, -18),
        TaperedEval(49, -27),
        TaperedEval(-6, -8),
        TaperedEval(-114, 45),
        TaperedEval(-84, 61),
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
        TaperedEval(-49, -41),
        TaperedEval(-4, -33),
        TaperedEval(-37, -17),
        TaperedEval(-2, 6),
        TaperedEval(4, 2),
        TaperedEval(-9, -5),
        TaperedEval(-11, -17),
        TaperedEval(-62, -39),
    ],
    [
        TaperedEval(-25, -18),
        TaperedEval(-19, 4),
        TaperedEval(-11, -12),
        TaperedEval(1, 1),
        TaperedEval(7, 7),
        TaperedEval(3, -13),
        TaperedEval(-7, 8),
        TaperedEval(-3, -3),
    ],
    [
        TaperedEval(-13, -36),
        TaperedEval(-1, -7),
        TaperedEval(-2, 5),
        TaperedEval(15, 30),
        TaperedEval(12, 23),
        TaperedEval(9, -2),
        TaperedEval(8, -2),
        TaperedEval(1, -19),
    ],
    [
        TaperedEval(0, 1),
        TaperedEval(13, 14),
        TaperedEval(18, 45),
        TaperedEval(14, 51),
        TaperedEval(16, 50),
        TaperedEval(17, 51),
        TaperedEval(42, 25),
        TaperedEval(12, 19),
    ],
    [
        TaperedEval(13, 0),
        TaperedEval(25, 18),
        TaperedEval(35, 46),
        TaperedEval(40, 59),
        TaperedEval(30, 64),
        TaperedEval(46, 55),
        TaperedEval(27, 38),
        TaperedEval(41, 19),
    ],
    [
        TaperedEval(-27, -13),
        TaperedEval(23, 8),
        TaperedEval(30, 37),
        TaperedEval(44, 38),
        TaperedEval(62, 35),
        TaperedEval(54, 52),
        TaperedEval(7, 18),
        TaperedEval(-14, 3),
    ],
    [
        TaperedEval(-4, -14),
        TaperedEval(2, -7),
        TaperedEval(38, -8),
        TaperedEval(61, 20),
        TaperedEval(46, 20),
        TaperedEval(43, -21),
        TaperedEval(-26, 1),
        TaperedEval(12, -9),
    ],
    [
        TaperedEval(-133, -76),
        TaperedEval(-14, -17),
        TaperedEval(-54, 15),
        TaperedEval(-2, 5),
        TaperedEval(-5, 3),
        TaperedEval(-32, 14),
        TaperedEval(-2, -1),
        TaperedEval(-42, -46),
    ],
];
pub const BISHOPS: SquareTable = [
    [
        TaperedEval(21, -14),
        TaperedEval(19, -7),
        TaperedEval(8, -15),
        TaperedEval(-8, -1),
        TaperedEval(-6, -3),
        TaperedEval(8, 2),
        TaperedEval(16, -14),
        TaperedEval(29, -11),
    ],
    [
        TaperedEval(16, -4),
        TaperedEval(21, -19),
        TaperedEval(22, -11),
        TaperedEval(7, 4),
        TaperedEval(15, -1),
        TaperedEval(22, -11),
        TaperedEval(36, -10),
        TaperedEval(29, -28),
    ],
    [
        TaperedEval(10, -4),
        TaperedEval(18, 5),
        TaperedEval(14, 8),
        TaperedEval(16, 9),
        TaperedEval(21, 13),
        TaperedEval(24, 8),
        TaperedEval(24, -3),
        TaperedEval(19, -1),
    ],
    [
        TaperedEval(-4, -4),
        TaperedEval(14, 1),
        TaperedEval(17, 17),
        TaperedEval(36, 19),
        TaperedEval(28, 16),
        TaperedEval(5, 18),
        TaperedEval(6, 7),
        TaperedEval(11, -14),
    ],
    [
        TaperedEval(0, -3),
        TaperedEval(27, 12),
        TaperedEval(21, 13),
        TaperedEval(58, 22),
        TaperedEval(39, 28),
        TaperedEval(37, 11),
        TaperedEval(21, 23),
        TaperedEval(-7, 14),
    ],
    [
        TaperedEval(13, 2),
        TaperedEval(30, 16),
        TaperedEval(34, 15),
        TaperedEval(26, 10),
        TaperedEval(46, 12),
        TaperedEval(37, 31),
        TaperedEval(12, 26),
        TaperedEval(-19, 19),
    ],
    [
        TaperedEval(-43, 14),
        TaperedEval(13, 10),
        TaperedEval(-7, 13),
        TaperedEval(-34, 20),
        TaperedEval(-39, 23),
        TaperedEval(-26, 22),
        TaperedEval(-48, 30),
        TaperedEval(-48, 20),
    ],
    [
        TaperedEval(-43, 18),
        TaperedEval(-33, 18),
        TaperedEval(-59, 21),
        TaperedEval(-67, 26),
        TaperedEval(-53, 14),
        TaperedEval(-78, 6),
        TaperedEval(-4, 13),
        TaperedEval(-27, 15),
    ],
];
pub const ROOKS: SquareTable = [
    [
        TaperedEval(-18, -20),
        TaperedEval(-4, -11),
        TaperedEval(1, -14),
        TaperedEval(9, -18),
        TaperedEval(2, -20),
        TaperedEval(1, -7),
        TaperedEval(4, -13),
        TaperedEval(-10, -34),
    ],
    [
        TaperedEval(-49, -17),
        TaperedEval(-19, -24),
        TaperedEval(-12, -12),
        TaperedEval(-6, -15),
        TaperedEval(-12, -17),
        TaperedEval(-1, -27),
        TaperedEval(5, -28),
        TaperedEval(-44, -18),
    ],
    [
        TaperedEval(-35, -6),
        TaperedEval(-21, -1),
        TaperedEval(-28, 4),
        TaperedEval(-13, -5),
        TaperedEval(-21, -3),
        TaperedEval(-16, -2),
        TaperedEval(5, -3),
        TaperedEval(-17, -13),
    ],
    [
        TaperedEval(-23, 8),
        TaperedEval(-23, 26),
        TaperedEval(-20, 26),
        TaperedEval(-13, 21),
        TaperedEval(-13, 15),
        TaperedEval(-11, 20),
        TaperedEval(7, 16),
        TaperedEval(-12, 8),
    ],
    [
        TaperedEval(-2, 22),
        TaperedEval(13, 23),
        TaperedEval(12, 28),
        TaperedEval(43, 18),
        TaperedEval(20, 21),
        TaperedEval(31, 17),
        TaperedEval(22, 12),
        TaperedEval(17, 18),
    ],
    [
        TaperedEval(-7, 35),
        TaperedEval(47, 17),
        TaperedEval(24, 33),
        TaperedEval(45, 15),
        TaperedEval(43, 22),
        TaperedEval(31, 31),
        TaperedEval(69, 6),
        TaperedEval(30, 21),
    ],
    [
        TaperedEval(9, 41),
        TaperedEval(2, 47),
        TaperedEval(22, 47),
        TaperedEval(41, 45),
        TaperedEval(2, 62),
        TaperedEval(43, 23),
        TaperedEval(1, 36),
        TaperedEval(52, 22),
    ],
    [
        TaperedEval(36, 33),
        TaperedEval(31, 42),
        TaperedEval(9, 55),
        TaperedEval(9, 48),
        TaperedEval(-7, 53),
        TaperedEval(12, 54),
        TaperedEval(44, 43),
        TaperedEval(62, 39),
    ],
];
pub const QUEENS: SquareTable = [
    [
        TaperedEval(14, -56),
        TaperedEval(19, -74),
        TaperedEval(22, -74),
        TaperedEval(35, -44),
        TaperedEval(33, -76),
        TaperedEval(-3, -57),
        TaperedEval(-7, -52),
        TaperedEval(8, -23),
    ],
    [
        TaperedEval(21, -56),
        TaperedEval(20, -35),
        TaperedEval(25, -66),
        TaperedEval(16, -31),
        TaperedEval(19, -41),
        TaperedEval(40, -97),
        TaperedEval(47, -113),
        TaperedEval(24, -41),
    ],
    [
        TaperedEval(12, -48),
        TaperedEval(18, -23),
        TaperedEval(16, 11),
        TaperedEval(8, -15),
        TaperedEval(13, -4),
        TaperedEval(10, 18),
        TaperedEval(32, -20),
        TaperedEval(32, -28),
    ],
    [
        TaperedEval(11, -31),
        TaperedEval(16, 10),
        TaperedEval(6, 10),
        TaperedEval(-6, 66),
        TaperedEval(-3, 60),
        TaperedEval(13, 59),
        TaperedEval(16, 40),
        TaperedEval(28, 61),
    ],
    [
        TaperedEval(4, -15),
        TaperedEval(-2, 34),
        TaperedEval(-9, 28),
        TaperedEval(-4, 81),
        TaperedEval(0, 102),
        TaperedEval(11, 115),
        TaperedEval(24, 109),
        TaperedEval(35, 80),
    ],
    [
        TaperedEval(-9, 10),
        TaperedEval(-4, 16),
        TaperedEval(-15, 46),
        TaperedEval(4, 65),
        TaperedEval(9, 106),
        TaperedEval(42, 124),
        TaperedEval(59, 102),
        TaperedEval(24, 114),
    ],
    [
        TaperedEval(-3, 23),
        TaperedEval(-56, 71),
        TaperedEval(-12, 61),
        TaperedEval(-25, 94),
        TaperedEval(-45, 154),
        TaperedEval(31, 99),
        TaperedEval(-33, 103),
        TaperedEval(33, 80),
    ],
    [
        TaperedEval(0, 29),
        TaperedEval(15, 55),
        TaperedEval(16, 61),
        TaperedEval(27, 69),
        TaperedEval(38, 82),
        TaperedEval(73, 86),
        TaperedEval(58, 88),
        TaperedEval(77, 78),
    ],
];
pub const KINGS: SquareTable = [
    [
        TaperedEval(-33, -73),
        TaperedEval(19, -43),
        TaperedEval(-27, -20),
        TaperedEval(-119, -24),
        TaperedEval(-49, -55),
        TaperedEval(-103, -14),
        TaperedEval(-10, -48),
        TaperedEval(-7, -109),
    ],
    [
        TaperedEval(-18, -19),
        TaperedEval(-28, -5),
        TaperedEval(-42, 5),
        TaperedEval(-104, 20),
        TaperedEval(-62, 10),
        TaperedEval(-69, 13),
        TaperedEval(-22, -15),
        TaperedEval(-10, -39),
    ],
    [
        TaperedEval(-30, -20),
        TaperedEval(-16, 2),
        TaperedEval(-2, 13),
        TaperedEval(-21, 32),
        TaperedEval(3, 24),
        TaperedEval(-10, 15),
        TaperedEval(-8, -7),
        TaperedEval(-50, -17),
    ],
    [
        TaperedEval(1, -23),
        TaperedEval(30, 18),
        TaperedEval(59, 29),
        TaperedEval(37, 48),
        TaperedEval(54, 36),
        TaperedEval(38, 29),
        TaperedEval(49, 4),
        TaperedEval(-28, -21),
    ],
    [
        TaperedEval(2, -2),
        TaperedEval(33, 37),
        TaperedEval(53, 41),
        TaperedEval(31, 46),
        TaperedEval(48, 44),
        TaperedEval(57, 40),
        TaperedEval(51, 29),
        TaperedEval(-26, -3),
    ],
    [
        TaperedEval(3, 3),
        TaperedEval(38, 47),
        TaperedEval(47, 40),
        TaperedEval(35, 32),
        TaperedEval(36, 23),
        TaperedEval(55, 51),
        TaperedEval(35, 49),
        TaperedEval(1, 1),
    ],
    [
        TaperedEval(-7, -44),
        TaperedEval(19, 32),
        TaperedEval(23, 26),
        TaperedEval(11, 7),
        TaperedEval(15, 10),
        TaperedEval(17, 17),
        TaperedEval(11, 42),
        TaperedEval(-4, -27),
    ],
    [
        TaperedEval(-9, -74),
        TaperedEval(1, -48),
        TaperedEval(3, -24),
        TaperedEval(3, -14),
        TaperedEval(2, -23),
        TaperedEval(-1, -18),
        TaperedEval(-1, -26),
        TaperedEval(-5, -61),
    ],
];

pub const PAWN_TABLE: SquareTable = add_piece(PAWNS, PAWN_CNT);
pub const KNIGHT_TABLE: SquareTable = add_piece(KNIGHTS, KNIGHT_CNT);
pub const BISHOP_TABLE: SquareTable = add_piece(BISHOPS, BISHOP_CNT);
pub const ROOK_TABLE: SquareTable = add_piece(ROOKS, ROOK_CNT);
pub const QUEEN_TABLE: SquareTable = add_piece(QUEENS, QUEEN_CNT);
pub const KING_TABLE: SquareTable = KINGS;
