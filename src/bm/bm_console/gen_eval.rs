use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};

use chess::{Board, MoveGen};
use rand::Rng;

use crate::bm::{
    bm_eval::eval::Evaluation,
    bm_runner::{
        ab_runner::AbRunner,
        config::{NoInfo, Run},
        time::{TimeManagementInfo, TimeManager},
    },
};

const RAND_MOVE_PROBABILITY: f32 = 0.1;

fn play_single(
    engine_0: &mut AbRunner,
    engine_1: &mut AbRunner,
    time_manager: &TimeManager,
    time_management_info: &[TimeManagementInfo],
) -> Vec<(Board, Evaluation)> {
    let mut evals = Vec::new();
    engine_0.set_board(Board::default());
    engine_1.set_board(Board::default());
    for _ in 0..160 {
        let mut move_gen = MoveGen::new_legal(engine_0.get_board());
        if move_gen.next().is_none() {
            break;
        }

        time_manager.initiate(engine_0.get_board(), time_management_info);
        let (mut make_move, eval, _, _) = engine_0.search::<Run, NoInfo>(1);
        time_manager.clear();
        let turn = match engine_0.get_board().side_to_move() {
            chess::Color::White => 1,
            chess::Color::Black => -1,
        };

        let mut move_gen = MoveGen::new_legal(engine_0.get_board());
        move_gen.set_iterator_mask(*engine_0.get_board().combined());
        if move_gen.next().is_none() {
            evals.push((*engine_0.get_board(), eval * turn));
        }

        if rand::thread_rng().gen::<f32>() < RAND_MOVE_PROBABILITY {
            let moves = MoveGen::new_legal(engine_0.get_board())
                .into_iter()
                .collect::<Box<_>>();
            make_move = moves[rand::thread_rng().gen_range(0..moves.len())];
        }
        engine_0.make_move(make_move);
        engine_1.make_move(make_move);

        std::mem::swap(engine_0, engine_1);
    }
    evals
}

fn gen_games(iter: usize) -> Vec<(Board, Evaluation)> {
    let mut evals = vec![];
    let time_management_options = TimeManagementInfo::MaxDepth(7);
    let time_manager = Arc::new(TimeManager::new());
    let mut engine_0 = AbRunner::new(Board::default(), time_manager.clone());
    let mut engine_1 = AbRunner::new(Board::default(), time_manager.clone());
    for i in 0..iter {
        println!("{}", i);

        evals.extend(play_single(
            &mut engine_0,
            &mut engine_1,
            &time_manager,
            &[time_management_options],
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
