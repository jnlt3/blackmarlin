use crate::*;

use super::common::*;

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
compile_error!("pext feature can only be enabled if target has BMI2.");

fn pext_u64(a: u64, mask: u64) -> u64 {
    // SAFETY: A compile error is raised if PEXT is not available. PEXT is always safe if available.
    unsafe { core::arch::x86_64::_pext_u64(a, mask) }
}

struct PextEntry {
    offset: u32,
    mask: BitBoard
}

const EMPTY_ENTRY: PextEntry = PextEntry {
    offset: 0,
    mask: BitBoard::EMPTY
};

struct PextIndexData {
    rook_data: [PextEntry; Square::NUM],
    bishop_data: [PextEntry; Square::NUM],
    table_size: usize
}

const INDEX_DATA: &PextIndexData = {
    let mut offset = 0;

    let mut rook_data = [EMPTY_ENTRY; Square::NUM];
    let mut i = 0;
    while i < rook_data.len() {
        let square = Square::index_const(i);
        let mask = get_rook_relevant_blockers(square);
        rook_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.len();
        i += 1;
    }

    let mut bishop_data = [EMPTY_ENTRY; Square::NUM];
    let mut i = 0;
    while i < bishop_data.len() {
        let square = Square::index_const(i);
        let mask = get_bishop_relevant_blockers(square);
        bishop_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.len();
        i += 1;
    }

    &PextIndexData {
        rook_data,
        bishop_data,
        table_size: offset as usize
    }
};

fn get_pext_index(index_data: &[PextEntry; Square::NUM], square: Square, blockers: BitBoard) -> usize {
    let index_data = &index_data[square as usize];
    let index = pext_u64(blockers.0, index_data.mask.0);
    index_data.offset as usize + index as usize
}

pub fn get_rook_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_pext_index(&INDEX_DATA.rook_data, square, blockers)
}

pub fn get_bishop_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_pext_index(&INDEX_DATA.bishop_data, square, blockers)
}

pub const SLIDING_MOVE_TABLE_SIZE: usize = INDEX_DATA.table_size;
