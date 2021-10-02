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
pub const TEMPO: T = TaperedEval(14, 10);
pub const DOUBLED: T = TaperedEval(-8, -17);
pub const ISOLATED: T = TaperedEval(-4, -17);
pub const CHAINED: T = TaperedEval(15, 13);
pub const THREAT: T = TaperedEval(32, 58);
pub const BISHOP_PAIR: T = TaperedEval(19, 88);
pub const PHALANX: T = TaperedEval(9, 9);
pub const PASSED_TABLE: RankTable = [
    TaperedEval(0, 0),
    TaperedEval(-6, 18),
    TaperedEval(-11, 23),
    TaperedEval(-3, 51),
    TaperedEval(17, 75),
    TaperedEval(39, 135),
    TaperedEval(100, 187),
    TaperedEval(0, 0),
];
pub const KNIGHT_MOBILITY: IndexTable<9> = [
    TaperedEval(14, 17),
    TaperedEval(20, 23),
    TaperedEval(25, 47),
    TaperedEval(23, 52),
    TaperedEval(22, 59),
    TaperedEval(31, 65),
    TaperedEval(29, 69),
    TaperedEval(37, 65),
    TaperedEval(36, 59),
];
pub const BISHOP_MOBILITY: IndexTable<14> = [
    TaperedEval(-11, -50),
    TaperedEval(-3, -34),
    TaperedEval(0, -9),
    TaperedEval(4, 9),
    TaperedEval(10, 28),
    TaperedEval(13, 44),
    TaperedEval(10, 53),
    TaperedEval(17, 66),
    TaperedEval(19, 74),
    TaperedEval(24, 77),
    TaperedEval(23, 72),
    TaperedEval(34, 62),
    TaperedEval(37, 72),
    TaperedEval(89, 38),
];
pub const ROOK_MOBILITY: IndexTable<15> = [
    TaperedEval(-1, 14),
    TaperedEval(1, 25),
    TaperedEval(-2, 27),
    TaperedEval(5, 42),
    TaperedEval(5, 50),
    TaperedEval(5, 51),
    TaperedEval(11, 63),
    TaperedEval(11, 66),
    TaperedEval(25, 71),
    TaperedEval(30, 76),
    TaperedEval(31, 81),
    TaperedEval(35, 84),
    TaperedEval(43, 85),
    TaperedEval(65, 73),
    TaperedEval(115, 55),
];
pub const QUEEN_MOBILITY: IndexTable<28> = [
    TaperedEval(20, -5),
    TaperedEval(20, 10),
    TaperedEval(13, 3),
    TaperedEval(20, 11),
    TaperedEval(20, 13),
    TaperedEval(17, 27),
    TaperedEval(11, 32),
    TaperedEval(15, 39),
    TaperedEval(16, 53),
    TaperedEval(19, 65),
    TaperedEval(20, 72),
    TaperedEval(26, 75),
    TaperedEval(28, 78),
    TaperedEval(25, 82),
    TaperedEval(28, 84),
    TaperedEval(28, 87),
    TaperedEval(31, 99),
    TaperedEval(35, 87),
    TaperedEval(35, 91),
    TaperedEval(38, 88),
    TaperedEval(44, 80),
    TaperedEval(50, 75),
    TaperedEval(47, 75),
    TaperedEval(60, 60),
    TaperedEval(65, 54),
    TaperedEval(50, 58),
    TaperedEval(38, 57),
    TaperedEval(30, 53),
];
pub const KNIGHT_ATTACK_CNT: T = TaperedEval(6, 9);
pub const BISHOP_ATTACK_CNT: T = TaperedEval(7, 16);
pub const ROOK_ATTACK_CNT: T = TaperedEval(32, -1);
pub const QUEEN_ATTACK_CNT: T = TaperedEval(7, 30);
pub const PAWN_CNT: T = TaperedEval(83, 155);
pub const KNIGHT_CNT: T = TaperedEval(236, 457);
pub const BISHOP_CNT: T = TaperedEval(267, 501);
pub const ROOK_CNT: T = TaperedEval(378, 863);
pub const QUEEN_CNT: T = TaperedEval(848, 1605);
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
        TaperedEval(-31, 5),
        TaperedEval(-23, -2),
        TaperedEval(-29, 3),
        TaperedEval(-27, 2),
        TaperedEval(-22, 14),
        TaperedEval(8, 5),
        TaperedEval(24, -15),
        TaperedEval(-4, -29),
    ],
    [
        TaperedEval(-27, -4),
        TaperedEval(-35, -5),
        TaperedEval(-25, -5),
        TaperedEval(-22, -9),
        TaperedEval(-13, -2),
        TaperedEval(-6, -6),
        TaperedEval(2, -22),
        TaperedEval(-12, -26),
    ],
    [
        TaperedEval(-29, 6),
        TaperedEval(-13, 4),
        TaperedEval(-9, -12),
        TaperedEval(-10, -22),
        TaperedEval(-1, -16),
        TaperedEval(-3, -12),
        TaperedEval(3, -9),
        TaperedEval(-15, -15),
    ],
    [
        TaperedEval(-13, 31),
        TaperedEval(-8, 21),
        TaperedEval(-7, 5),
        TaperedEval(13, -27),
        TaperedEval(25, -15),
        TaperedEval(29, -8),
        TaperedEval(14, 4),
        TaperedEval(-1, -4),
    ],
    [
        TaperedEval(-11, 64),
        TaperedEval(13, 49),
        TaperedEval(25, 24),
        TaperedEval(33, -17),
        TaperedEval(46, -26),
        TaperedEval(104, -8),
        TaperedEval(41, 16),
        TaperedEval(1, 32),
    ],
    [
        TaperedEval(63, 59),
        TaperedEval(53, 54),
        TaperedEval(69, 21),
        TaperedEval(71, -20),
        TaperedEval(71, -34),
        TaperedEval(8, -9),
        TaperedEval(-130, 54),
        TaperedEval(-104, 62),
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
        TaperedEval(-101, -58),
        TaperedEval(-21, -58),
        TaperedEval(-46, -16),
        TaperedEval(-22, 5),
        TaperedEval(-14, 2),
        TaperedEval(-11, -9),
        TaperedEval(-21, -50),
        TaperedEval(-98, -74),
    ],
    [
        TaperedEval(-38, -29),
        TaperedEval(-41, 7),
        TaperedEval(-14, -3),
        TaperedEval(-1, 13),
        TaperedEval(-1, 15),
        TaperedEval(-3, -8),
        TaperedEval(-13, 0),
        TaperedEval(-8, -21),
    ],
    [
        TaperedEval(-26, -39),
        TaperedEval(-2, 7),
        TaperedEval(2, 18),
        TaperedEval(15, 40),
        TaperedEval(19, 35),
        TaperedEval(8, 13),
        TaperedEval(8, 7),
        TaperedEval(-13, -26),
    ],
    [
        TaperedEval(-9, -7),
        TaperedEval(12, 24),
        TaperedEval(18, 56),
        TaperedEval(17, 63),
        TaperedEval(23, 60),
        TaperedEval(16, 60),
        TaperedEval(34, 35),
        TaperedEval(8, 5),
    ],
    [
        TaperedEval(6, 0),
        TaperedEval(22, 28),
        TaperedEval(43, 51),
        TaperedEval(42, 69),
        TaperedEval(35, 79),
        TaperedEval(57, 62),
        TaperedEval(37, 51),
        TaperedEval(34, 16),
    ],
    [
        TaperedEval(-26, -16),
        TaperedEval(24, 11),
        TaperedEval(47, 43),
        TaperedEval(67, 45),
        TaperedEval(109, 33),
        TaperedEval(110, 47),
        TaperedEval(58, 18),
        TaperedEval(40, -11),
    ],
    [
        TaperedEval(-10, -25),
        TaperedEval(2, -5),
        TaperedEval(43, -4),
        TaperedEval(67, 28),
        TaperedEval(63, 25),
        TaperedEval(101, -17),
        TaperedEval(-11, -2),
        TaperedEval(32, -22),
    ],
    [
        TaperedEval(-184, -83),
        TaperedEval(-28, -15),
        TaperedEval(-101, 32),
        TaperedEval(-1, 8),
        TaperedEval(22, 7),
        TaperedEval(-53, 28),
        TaperedEval(3, -2),
        TaperedEval(-92, -87),
    ],
];
pub const BISHOPS: SquareTable = [
    [
        TaperedEval(17, -15),
        TaperedEval(16, -1),
        TaperedEval(-10, -12),
        TaperedEval(-16, 7),
        TaperedEval(-16, 6),
        TaperedEval(-9, -1),
        TaperedEval(0, -10),
        TaperedEval(13, -17),
    ],
    [
        TaperedEval(11, -7),
        TaperedEval(14, -16),
        TaperedEval(14, -7),
        TaperedEval(-2, 3),
        TaperedEval(2, 4),
        TaperedEval(15, -11),
        TaperedEval(24, -12),
        TaperedEval(19, -30),
    ],
    [
        TaperedEval(2, -5),
        TaperedEval(15, 5),
        TaperedEval(7, 6),
        TaperedEval(10, 18),
        TaperedEval(10, 12),
        TaperedEval(10, 6),
        TaperedEval(13, -8),
        TaperedEval(22, -1),
    ],
    [
        TaperedEval(-6, -6),
        TaperedEval(11, 1),
        TaperedEval(12, 21),
        TaperedEval(29, 24),
        TaperedEval(28, 16),
        TaperedEval(7, 13),
        TaperedEval(11, 5),
        TaperedEval(16, -9),
    ],
    [
        TaperedEval(-7, 5),
        TaperedEval(26, 16),
        TaperedEval(20, 10),
        TaperedEval(56, 14),
        TaperedEval(39, 31),
        TaperedEval(36, 14),
        TaperedEval(19, 25),
        TaperedEval(4, 11),
    ],
    [
        TaperedEval(6, 1),
        TaperedEval(24, 18),
        TaperedEval(39, 11),
        TaperedEval(37, 9),
        TaperedEval(63, 9),
        TaperedEval(76, 20),
        TaperedEval(55, 15),
        TaperedEval(30, 10),
    ],
    [
        TaperedEval(-43, 15),
        TaperedEval(5, 12),
        TaperedEval(-2, 14),
        TaperedEval(-30, 16),
        TaperedEval(0, 13),
        TaperedEval(26, 11),
        TaperedEval(-19, 25),
        TaperedEval(13, 3),
    ],
    [
        TaperedEval(-63, 22),
        TaperedEval(-58, 28),
        TaperedEval(-97, 29),
        TaperedEval(-95, 31),
        TaperedEval(-60, 25),
        TaperedEval(-91, 19),
        TaperedEval(14, 14),
        TaperedEval(-44, 26),
    ],
];
pub const ROOKS: SquareTable = [
    [
        TaperedEval(-26, -13),
        TaperedEval(-15, -11),
        TaperedEval(-12, -6),
        TaperedEval(-5, -4),
        TaperedEval(-7, -16),
        TaperedEval(-14, 3),
        TaperedEval(-2, -13),
        TaperedEval(-16, -31),
    ],
    [
        TaperedEval(-63, -15),
        TaperedEval(-25, -21),
        TaperedEval(-28, -8),
        TaperedEval(-24, -12),
        TaperedEval(-26, -12),
        TaperedEval(-10, -25),
        TaperedEval(-3, -25),
        TaperedEval(-57, -13),
    ],
    [
        TaperedEval(-43, -6),
        TaperedEval(-32, 4),
        TaperedEval(-37, 5),
        TaperedEval(-20, 2),
        TaperedEval(-29, 0),
        TaperedEval(-30, 2),
        TaperedEval(-2, -5),
        TaperedEval(-24, -13),
    ],
    [
        TaperedEval(-35, 11),
        TaperedEval(-37, 29),
        TaperedEval(-29, 28),
        TaperedEval(-24, 21),
        TaperedEval(-24, 19),
        TaperedEval(-14, 17),
        TaperedEval(-4, 18),
        TaperedEval(-20, 4),
    ],
    [
        TaperedEval(-16, 23),
        TaperedEval(-6, 23),
        TaperedEval(6, 26),
        TaperedEval(27, 24),
        TaperedEval(14, 23),
        TaperedEval(32, 21),
        TaperedEval(21, 19),
        TaperedEval(20, 17),
    ],
    [
        TaperedEval(-8, 33),
        TaperedEval(32, 17),
        TaperedEval(16, 32),
        TaperedEval(45, 20),
        TaperedEval(72, 10),
        TaperedEval(69, 21),
        TaperedEval(104, 1),
        TaperedEval(42, 16),
    ],
    [
        TaperedEval(10, 37),
        TaperedEval(3, 47),
        TaperedEval(32, 42),
        TaperedEval(44, 43),
        TaperedEval(41, 48),
        TaperedEval(86, 17),
        TaperedEval(27, 35),
        TaperedEval(72, 18),
    ],
    [
        TaperedEval(25, 41),
        TaperedEval(30, 42),
        TaperedEval(11, 53),
        TaperedEval(20, 47),
        TaperedEval(23, 46),
        TaperedEval(54, 45),
        TaperedEval(78, 36),
        TaperedEval(89, 32),
    ],
];
pub const QUEENS: SquareTable = [
    [
        TaperedEval(12, -49),
        TaperedEval(6, -60),
        TaperedEval(8, -56),
        TaperedEval(12, -37),
        TaperedEval(17, -67),
        TaperedEval(-11, -50),
        TaperedEval(-3, -81),
        TaperedEval(-1, -36),
    ],
    [
        TaperedEval(10, -58),
        TaperedEval(18, -37),
        TaperedEval(19, -59),
        TaperedEval(15, -23),
        TaperedEval(10, -32),
        TaperedEval(32, -93),
        TaperedEval(43, -115),
        TaperedEval(21, -64),
    ],
    [
        TaperedEval(12, -48),
        TaperedEval(14, -21),
        TaperedEval(10, 11),
        TaperedEval(0, -14),
        TaperedEval(6, -11),
        TaperedEval(6, 10),
        TaperedEval(25, -29),
        TaperedEval(29, -33),
    ],
    [
        TaperedEval(5, -33),
        TaperedEval(5, 11),
        TaperedEval(6, 8),
        TaperedEval(-10, 61),
        TaperedEval(-12, 53),
        TaperedEval(12, 53),
        TaperedEval(15, 33),
        TaperedEval(24, 48),
    ],
    [
        TaperedEval(-4, -10),
        TaperedEval(-6, 33),
        TaperedEval(-10, 31),
        TaperedEval(-11, 80),
        TaperedEval(-8, 102),
        TaperedEval(10, 122),
        TaperedEval(24, 106),
        TaperedEval(25, 78),
    ],
    [
        TaperedEval(-7, 5),
        TaperedEval(-3, 14),
        TaperedEval(-9, 44),
        TaperedEval(2, 64),
        TaperedEval(22, 104),
        TaperedEval(66, 127),
        TaperedEval(81, 95),
        TaperedEval(42, 107),
    ],
    [
        TaperedEval(1, 18),
        TaperedEval(-52, 65),
        TaperedEval(-1, 59),
        TaperedEval(-36, 112),
        TaperedEval(-13, 149),
        TaperedEval(50, 103),
        TaperedEval(-46, 156),
        TaperedEval(36, 94),
    ],
    [
        TaperedEval(-11, 40),
        TaperedEval(5, 66),
        TaperedEval(13, 67),
        TaperedEval(29, 74),
        TaperedEval(50, 82),
        TaperedEval(106, 71),
        TaperedEval(62, 90),
        TaperedEval(83, 77),
    ],
];
pub const KINGS: SquareTable = [
    [
        TaperedEval(-26, -66),
        TaperedEval(14, -35),
        TaperedEval(-30, -20),
        TaperedEval(-126, -17),
        TaperedEval(-54, -64),
        TaperedEval(-113, -11),
        TaperedEval(-8, -42),
        TaperedEval(2, -103),
    ],
    [
        TaperedEval(-17, -17),
        TaperedEval(-29, 1),
        TaperedEval(-55, 15),
        TaperedEval(-117, 24),
        TaperedEval(-79, 15),
        TaperedEval(-87, 19),
        TaperedEval(-27, -2),
        TaperedEval(-3, -36),
    ],
    [
        TaperedEval(-45, -11),
        TaperedEval(-38, 15),
        TaperedEval(-37, 24),
        TaperedEval(-57, 43),
        TaperedEval(-41, 39),
        TaperedEval(-46, 25),
        TaperedEval(-25, 3),
        TaperedEval(-58, -12),
    ],
    [
        TaperedEval(10, -23),
        TaperedEval(30, 24),
        TaperedEval(42, 40),
        TaperedEval(-1, 55),
        TaperedEval(7, 48),
        TaperedEval(-7, 40),
        TaperedEval(23, 16),
        TaperedEval(-46, -14),
    ],
    [
        TaperedEval(9, 1),
        TaperedEval(50, 37),
        TaperedEval(85, 43),
        TaperedEval(23, 51),
        TaperedEval(46, 48),
        TaperedEval(51, 45),
        TaperedEval(52, 33),
        TaperedEval(-55, 10),
    ],
    [
        TaperedEval(8, 9),
        TaperedEval(75, 47),
        TaperedEval(93, 40),
        TaperedEval(74, 31),
        TaperedEval(71, 26),
        TaperedEval(107, 53),
        TaperedEval(62, 52),
        TaperedEval(4, 2),
    ],
    [
        TaperedEval(-9, -51),
        TaperedEval(45, 31),
        TaperedEval(56, 24),
        TaperedEval(34, 6),
        TaperedEval(38, 9),
        TaperedEval(51, 20),
        TaperedEval(12, 47),
        TaperedEval(-4, -23),
    ],
    [
        TaperedEval(-14, -174),
        TaperedEval(25, -75),
        TaperedEval(16, -44),
        TaperedEval(19, -18),
        TaperedEval(14, -39),
        TaperedEval(4, -33),
        TaperedEval(7, -39),
        TaperedEval(-7, -138),
    ],
];

pub const PAWN_TABLE: SquareTable = add_piece(PAWNS, PAWN_CNT);
pub const KNIGHT_TABLE: SquareTable = add_piece(KNIGHTS, KNIGHT_CNT);
pub const BISHOP_TABLE: SquareTable = add_piece(BISHOPS, BISHOP_CNT);
pub const ROOK_TABLE: SquareTable = add_piece(ROOKS, ROOK_CNT);
pub const QUEEN_TABLE: SquareTable = add_piece(QUEENS, QUEEN_CNT);
pub const KING_TABLE: SquareTable = KINGS;
