use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
    time::Duration,
};

use chess::{Board, MoveGen};
use rand::Rng;

use crate::bm::{
    bm_eval::eval::Evaluation,
    bm_runner::{
        ab_runner::AbRunner,
        config::{NoInfo, Run},
        time::{ConstDepth, TimeManager},
    },
};

const RAND_MOVE_PROBABILITY: f32 = 0.1;

fn play_single(
    mut engine_0: AbRunner,
    mut engine_1: AbRunner,
    time_manager_0: &ConstDepth,
    time_manager_1: &ConstDepth,
) -> Vec<(Board, Evaluation)> {
    let mut evals = Vec::new();
    engine_0.set_board(Board::default());
    engine_1.set_board(Board::default());
    for i in 0..80 {
        let (engine, time_manager, other_engine) = if i % 2 == 0 {
            (&mut engine_0, time_manager_0, &mut engine_1)
        } else {
            (&mut engine_1, time_manager_1, &mut engine_0)
        };

        let mut move_gen = MoveGen::new_legal(engine.get_board());
        if move_gen.next().is_none() {
            break;
        }

        time_manager.initiate(Duration::default(), engine.get_board());
        let (mut make_move, eval, _, _) = engine.search::<Run, NoInfo>(1);
        time_manager.clear();
        let turn = match engine.get_board().side_to_move() {
            chess::Color::White => 1,
            chess::Color::Black => -1,
        };

        let mut move_gen = MoveGen::new_legal(engine.get_board());
        move_gen.set_iterator_mask(*engine.get_board().combined());
        if move_gen.next().is_none() {
            evals.push((*engine.get_board(), eval * turn));
        }

        if rand::thread_rng().gen::<f32>() < RAND_MOVE_PROBABILITY {
            let moves = MoveGen::new_legal(engine.get_board())
                .into_iter()
                .collect::<Box<_>>();
            make_move = moves[rand::thread_rng().gen_range(0..moves.len())];
        }
        engine.make_move(make_move);
        other_engine.make_move(make_move);
    }
    evals
}

fn gen_games(iter: usize) -> Vec<(Board, Evaluation)> {
    let mut evals = vec![];
    for i in 0..iter {
        println!("{}", i);
        let time_manager_0 = Arc::new(ConstDepth::new(7));
        let engine_0 = AbRunner::new(Board::default(), time_manager_0.clone());

        let time_manager_1 = Arc::new(ConstDepth::new(7));
        let engine_1 = AbRunner::new(Board::default(), time_manager_1.clone());

        evals.extend(play_single(
            engine_0,
            engine_1,
            &time_manager_0,
            &time_manager_1,
        ));
    }
    evals
}

pub fn gen_eval() {
    for _ in 0.. {
        let mut evals = vec![];
        let mut threads = vec![];
        for _ in 0..4 {
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
