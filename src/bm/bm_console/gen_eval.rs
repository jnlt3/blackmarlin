use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
    time::Duration,
};

use crate::bm::bm_eval::eval::Depth::Next;
use chess::{Board, MoveGen, EMPTY};
use rand::Rng;

use crate::bm::{
    bm_eval::{eval::Evaluation, evaluator::StdEvaluator},
    bm_runner::{
        ab_runner::AbRunner,
        config::{NoInfo, Run},
        time::{ConstDepth, TimeManager},
    },
};

const RAND_MOVE_PROBABILITY: f32 = 0.1;

fn play_single(engine: &mut AbRunner, time_manager: &ConstDepth) -> Vec<(Board, Evaluation)> {
    let mut evals = Vec::new();
    engine.set_board(Board::default());
    for _ in 0..160 {
        time_manager.initiate(Duration::default(), 0);
        let mut move_gen = MoveGen::new_legal(engine.get_board());
        if move_gen.next().is_none() {
            break;
        }
        let (mut make_move, eval, _, _) = engine.search::<Run, NoInfo>(1);
        let turn = match engine.get_board().side_to_move() {
            chess::Color::White => 1,
            chess::Color::Black => -1,
        };

        let mut move_gen = MoveGen::new_legal(engine.get_board());
        move_gen.set_iterator_mask(*engine.get_board().combined());
        if move_gen.next().is_none() {
            evals.push((*engine.get_board(), eval * turn));
        }
        time_manager.clear();

        if rand::thread_rng().gen::<f32>() < RAND_MOVE_PROBABILITY {
            let moves = MoveGen::new_legal(engine.get_board())
                .into_iter()
                .collect::<Box<_>>();
            make_move = moves[rand::thread_rng().gen_range(0..moves.len())];
        }
        engine.make_move(make_move)
    }
    evals
}

fn gen_games(iter: usize) -> Vec<(Board, Evaluation)> {
    let time_manager = Arc::new(ConstDepth::new(8));
    let mut engine = AbRunner::new(Board::default(), time_manager.clone());
    let mut evals = vec![];
    for i in 0..iter {
        println!("{}", i);
        evals.extend(play_single(&mut engine, &time_manager));
    }
    evals
}

pub fn gen_eval() {
    for _ in 0.. {
        let mut evals = vec![];
        let mut threads = vec![];
        for _ in 0..6 {
            threads.push(std::thread::spawn(move || gen_games(100)))
        }
        for t in threads {
            evals.extend(t.join().unwrap());
        }
        let mut output = String::new();
        for (board, eval) in evals {
            output += &format!("{} [{}]\n", &board.to_string(), eval.raw());
        }
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("./data/quiet_01.txt")
            .unwrap();
        let mut write = BufWriter::new(file);
        write.write(output.as_bytes()).unwrap();
    }
}

fn small_q_search(
    board: Board,
    depth: u8,
    mut alpha: Evaluation,
    beta: Evaluation,
    eval: &mut StdEvaluator,
) -> (Evaluation, Board) {
    if depth == 0 {
        return (eval.evaluate(&board), board);
    }

    let mut best_board = board;

    if *board.checkers() == EMPTY {
        let stand_pat = eval.evaluate(&board);
        if stand_pat > alpha {
            alpha = stand_pat;
            if stand_pat >= beta {
                return (stand_pat, best_board);
            }
        }
    }

    let mut best = None;
    let mut move_gen = MoveGen::new_legal(&board);
    if *board.checkers() != EMPTY {
        move_gen.set_iterator_mask(*board.combined());
    }

    for make_move in move_gen {
        let new_board = board.make_move_new(make_move);

        let (eval, next_best) =
            small_q_search(new_board, depth - 1, beta >> Next, alpha >> Next, eval);
        let eval = eval << Next;
        if best.is_none() || eval > best.unwrap() {
            best = Some(eval);
        }
        if eval > alpha {
            alpha = eval;
            best_board = next_best;
            if eval >= beta {
                return (eval, best_board);
            }
        }
    }

    (best.unwrap_or(alpha), best_board)
}
