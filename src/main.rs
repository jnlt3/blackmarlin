use std::str::FromStr;

use crate::bm::{bm_console::BmConsole, policy};
use cozy_chess::Board;
use text_io::read;

mod bm;

fn main() {
    /*
    let start_pos = Board::from_fen(
        "rnb1kbnr/pppppppp/8/6q1/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 1 1",
        false,
    )
    .unwrap();
    let mut moves = vec![];
    start_pos.generate_moves(|piece_moves| {
        for mv in piece_moves {
            moves.push(mv);
        }
        false
    });
    for mv in moves {
        println!("{}: {}", mv, policy::move_eval(&start_pos, mv));
    }
    */

    let mut bm_console = BmConsole::new();
    for arg in std::env::args() {
        if arg.trim() == "bench" {
            bm_console.input("bench".to_string());
            return;
        }
    }
    while bm_console.input(read!("{}\n")) {}
}
