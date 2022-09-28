use crate::*;

use super::common::*;

struct BlackMagicEntry {
    neg_mask: BitBoard,
    magic: u64,
    offset: u32
}

const EMPTY_ENTRY: BlackMagicEntry = BlackMagicEntry {
    neg_mask: BitBoard::EMPTY,
    magic: 0,
    offset: 0
};

// const fns can't be parameterized by functions, so we use a macro instead.
macro_rules! gen_entries {
    ($relevant_blockers:ident, $raw_magics:expr) => {{
        let raw_magics = $raw_magics;
        let mut magics = [EMPTY_ENTRY; Square::NUM];
        let mut i = 0;
        while i < raw_magics.len() {
            let square = Square::index_const(i);
            let neg_mask = BitBoard(!$relevant_blockers(square).0);
            let (magic, offset) = raw_magics[i];
            magics[i] = BlackMagicEntry { neg_mask, magic, offset };
            i += 1;
        }
        magics
    }};
}

// Black magics found by Volker Annuss and Niklas Fiekas
// http://talkchess.com/forum/viewtopic.php?t=64790

const ROOK_MAGICS: &[BlackMagicEntry; Square::NUM] = &gen_entries!(
    get_rook_relevant_blockers,
    [
        (0x80280013FF84FFFF, 10890), (0x5FFBFEFDFEF67FFF, 50579), (0xFFEFFAFFEFFDFFFF, 62020),
        (0x003000900300008A, 67322), (0x0050028010500023, 80251), (0x0020012120A00020, 58503),
        (0x0030006000C00030, 51175), (0x0058005806B00002, 83130), (0x7FBFF7FBFBEAFFFC, 50430),
        (0x0000140081050002, 21613), (0x0000180043800048, 72625), (0x7FFFE800021FFFB8, 80755),
        (0xFFFFCFFE7FCFFFAF, 69753), (0x00001800C0180060, 26973), (0x4F8018005FD00018, 84972),
        (0x0000180030620018, 31958), (0x00300018010C0003, 69272), (0x0003000C0085FFFF, 48372),
        (0xFFFDFFF7FBFEFFF7, 65477), (0x7FC1FFDFFC001FFF, 43972), (0xFFFEFFDFFDFFDFFF, 57154),
        (0x7C108007BEFFF81F, 53521), (0x20408007BFE00810, 30534), (0x0400800558604100, 16548),
        (0x0040200010080008, 46407), (0x0010020008040004, 11841), (0xFFFDFEFFF7FBFFF7, 21112),
        (0xFEBF7DFFF8FEFFF9, 44214), (0xC00000FFE001FFE0, 57925), (0x4AF01F00078007C3, 29574),
        (0xBFFBFAFFFB683F7F, 17309), (0x0807F67FFA102040, 40143), (0x200008E800300030, 64659),
        (0x0000008780180018, 70469), (0x0000010300180018, 62917), (0x4000008180180018, 60997),
        (0x008080310005FFFA, 18554), (0x4000188100060006, 14385), (0xFFFFFF7FFFBFBFFF,     0),
        (0x0000802000200040, 38091), (0x20000202EC002800, 25122), (0xFFFFF9FF7CFFF3FF, 60083),
        (0x000000404B801800, 72209), (0x2000002FE03FD000, 67875), (0xFFFFFF6FFE7FCFFD, 56290),
        (0xBFF7EFFFBFC00FFF, 43807), (0x000000100800A804, 73365), (0x6054000A58005805, 76398),
        (0x0829000101150028, 20024), (0x00000085008A0014,  9513), (0x8000002B00408028, 24324),
        (0x4000002040790028, 22996), (0x7800002010288028, 23213), (0x0000001800E08018, 56002),
        (0xA3A80003F3A40048, 22809), (0x2003D80000500028, 44545), (0xFFFFF37EEFEFDFBE, 36072),
        (0x40000280090013C1,  4750), (0xBF7FFEFFBFFAF71F,  6014), (0xFFFDFFFF777B7D6E, 36054),
        (0x48300007E8080C02, 78538), (0xAFE0000FFF780402, 28745), (0xEE73FFFBFFBB77FE,  8555),
        (0x0002000308482882,  1009)
    ]
);

const BISHOP_MAGICS: &[BlackMagicEntry; Square::NUM] = &gen_entries!(
    get_bishop_relevant_blockers,
    [
        (0xA7020080601803D8, 60984), (0x13802040400801F1, 66046), (0x0A0080181001F60C, 32910),
        (0x1840802004238008, 16369), (0xC03FE00100000000, 42115), (0x24C00BFFFF400000,   835),
        (0x0808101F40007F04, 18910), (0x100808201EC00080, 25911), (0xFFA2FEFFBFEFB7FF, 63301),
        (0x083E3EE040080801, 16063), (0xC0800080181001F8, 17481), (0x0440007FE0031000, 59361),
        (0x2010007FFC000000, 18735), (0x1079FFE000FF8000, 61249), (0x3C0708101F400080, 68938),
        (0x080614080FA00040, 61791), (0x7FFE7FFF817FCFF9, 21893), (0x7FFEBFFFA01027FD, 62068),
        (0x53018080C00F4001, 19829), (0x407E0001000FFB8A, 26091), (0x201FE000FFF80010, 15815),
        (0xFFDFEFFFDE39FFEF, 16419), (0xCC8808000FBF8002, 59777), (0x7FF7FBFFF8203FFF, 16288),
        (0x8800013E8300C030, 33235), (0x0420009701806018, 15459), (0x7FFEFF7F7F01F7FD, 15863),
        (0x8700303010C0C006, 75555), (0xC800181810606000, 79445), (0x20002038001C8010, 15917),
        (0x087FF038000FC001,  8512), (0x00080C0C00083007, 73069), (0x00000080FC82C040, 16078),
        (0x000000407E416020, 19168), (0x00600203F8008020, 11056), (0xD003FEFE04404080, 62544),
        (0xA00020C018003088, 80477), (0x7FBFFE700BFFE800, 75049), (0x107FF00FE4000F90, 32947),
        (0x7F8FFFCFF1D007F8, 59172), (0x0000004100F88080, 55845), (0x00000020807C4040, 61806),
        (0x00000041018700C0, 73601), (0x0010000080FC4080, 15546), (0x1000003C80180030, 45243),
        (0xC10000DF80280050, 20333), (0xFFFFFFBFEFF80FDC, 33402), (0x000000101003F812, 25917),
        (0x0800001F40808200, 32875), (0x084000101F3FD208,  4639), (0x080000000F808081, 17077),
        (0x0004000008003F80, 62324), (0x08000001001FE040, 18159), (0x72DD000040900A00, 61436),
        (0xFFFFFEFFBFEFF81D, 57073), (0xCD8000200FEBF209, 61025), (0x100000101EC10082, 81259),
        (0x7FBAFFFFEFE0C02F, 64083), (0x7F83FFFFFFF07F7F, 56114), (0xFFF1FFFFFFF7FFC1, 57058),
        (0x0878040000FFE01F, 58912), (0x945E388000801012, 22194), (0x0840800080200FDA, 70880),
        (0x100000C05F582008, 11140)
    ]
);

const ROOK_INDEX_BITS: usize = 12;

const BISHOP_INDEX_BITS: usize = 9;

const fn get_magic_index(magics: &[BlackMagicEntry; Square::NUM], index_bits: usize, square: Square, blockers: BitBoard) -> usize {
    let magic = &magics[square as usize];
    let relevant_blockers = blockers.0 | magic.neg_mask.0;
    let hash = relevant_blockers.wrapping_mul(magic.magic);
    magic.offset as usize + (hash >> (Square::NUM - index_bits)) as usize
}

pub const fn get_rook_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_magic_index(ROOK_MAGICS, ROOK_INDEX_BITS, square, blockers)
}

pub const fn get_bishop_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_magic_index(BISHOP_MAGICS, BISHOP_INDEX_BITS, square, blockers)
}

pub const SLIDING_MOVE_TABLE_SIZE: usize = 87988;
