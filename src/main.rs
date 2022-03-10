use std::str::FromStr;

use crate::bm::{bm_console::BmConsole, policy};
use cozy_chess::Board;
use text_io::read;

mod bm;

fn main() {
    /*
    let mut board = Board::from_fen("3k4/7P/8/8/8/8/8/7K w - - 0 1", false).unwrap();

    for i in 0..1 {
        let mut moves = vec![];
        board.generate_moves(|piece_moves| {
            for mv in piece_moves {
                moves.push(mv);
            }
            false
        });
        let mut highest = i16::MIN;
        let mut best_move = None;
        for mv in moves {
            let score = policy::move_eval(&board, mv);
            println!("{} {}", mv, score);
            if score > highest {
                best_move = Some(mv);
                highest = score;
            }
        }
        board.play_unchecked(best_move.unwrap());
        println!("{}", best_move.unwrap());
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
