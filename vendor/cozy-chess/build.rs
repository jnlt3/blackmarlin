use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use cozy_chess_types::*;

fn write_moves(
    table: &mut [BitBoard],
    relevant_blockers: impl Fn(Square) -> BitBoard,
    table_index: impl Fn(Square, BitBoard) -> usize,
    slider_moves: impl Fn(Square, BitBoard) -> BitBoard
) {
    for &square in &Square::ALL {
        let mask = relevant_blockers(square);
        for blockers in mask.iter_subsets() {
            table[table_index(square, blockers)] = slider_moves(square, blockers);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut table = [BitBoard::EMPTY; SLIDING_MOVE_TABLE_SIZE];
    write_moves(
        &mut table,
        get_rook_relevant_blockers,
        get_rook_moves_index,
        get_rook_moves_slow
    );
    write_moves(
        &mut table,
        get_bishop_relevant_blockers,
        get_bishop_moves_index,
        get_bishop_moves_slow
    );

    let mut out_file: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_file.push("sliding_moves.rs");
    let mut out_file = BufWriter::new(File::create(out_file).unwrap());
    write!(&mut out_file, "const SLIDING_MOVES: &[u64; {}] = &[", table.len()).unwrap();
    for magic in &table {
        write!(&mut out_file, "{},", magic.0).unwrap();
    }
    write!(&mut out_file, "];").unwrap();
}
