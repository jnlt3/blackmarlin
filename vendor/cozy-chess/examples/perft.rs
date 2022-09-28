use std::time::Instant;
use std::env::args;

use cozy_chess::*;

fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut nodes = 0;
        board.generate_moves(|moves| {
            for mv in moves {
                let mut board = board.clone();
                board.play_unchecked(mv);
                nodes += perft(&board, depth - 1);
            }
            false
        });
        nodes
    }
}

fn perft_bulk(board: &Board, depth: u8) -> u64 {
    let mut nodes = 0;
    match depth {
        0 => nodes += 1,
        1 => {
            board.generate_moves(|moves| {
                nodes += moves.len() as u64;
                false
            });
        }
        _ => {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    let child_nodes = perft_bulk(&board, depth - 1);
                    nodes += child_nodes;
                }
                false
            });
        }
    }
    nodes
}

fn help_message() {
    eprintln!("USAGE: perft <depth> [<FEN>] [--no-bulk] [--help]");
    eprintln!("  Defaults to the start position if no FEN is specified.");
    eprintln!("  OPTIONS:");
    eprintln!("    --no-bulk: Disable bulk counting on leaf node parents.");
    eprintln!("    --help:    Print this message.");
}

fn main() {
    let mut depth = None;
    let mut board = None;
    let mut bulk = true;
    let mut help = false;
    for arg in args().skip(1) {
        if arg == "--no-bulk" {
            bulk = false;
            continue;
        }
        if arg == "--help" {
            help = true;
            continue;
        }
        if depth.is_none() {
            if let Ok(arg) = arg.parse() {
                depth = Some(arg);
                continue;
            }
            eprintln!("ERROR: Invalid depth '{}'.", arg);
            help_message();
            return;
        }
        if board.is_none() {
            if let Ok(arg) = Board::from_fen(&arg, false) {
                board = Some(arg);
                continue;
            }
            if let Ok(arg) = Board::from_fen(&arg, true) {
                board = Some(arg);
                continue;
            }
            eprintln!("ERROR: Invalid FEN '{}'.", arg);
            help_message();
            return;
        }
        eprintln!("ERROR: Unexpected argument '{}'.", arg);
        help_message();
        return;
    }

    if help {
        help_message();
        return;
    }

    let depth = if let Some(depth) = depth {
        depth
    } else {
        eprintln!("ERROR: Missing required argument 'depth'.");
        help_message();
        return;
    };
    let board = board.unwrap_or_default();

    let start = Instant::now();
    let nodes = if bulk {
        perft_bulk(&board, depth)
    } else {
        perft(&board, depth)
    };
    let elapsed = start.elapsed();
    let nps = nodes as f64 / elapsed.as_secs_f64();
    println!("{} nodes in {:.2?} ({:.0} nps)", nodes, elapsed, nps);
}
