use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};

use arrayvec::ArrayVec;
use cozy_chess::{BitBoard, Board, Move};
use rand::Rng;

use crate::bm::{
    bm_eval::eval::Evaluation,
    bm_runner::{
        ab_runner::AbRunner,
        config::{NoInfo, Run},
        time::{TimeManagementInfo, TimeManager},
    },
};

fn play_single(
    engine_0: &mut AbRunner,
    engine_1: &mut AbRunner,
    time_manager: &TimeManager,
    time_management_info: &[TimeManagementInfo],
) -> Vec<(Board, Evaluation, f32)> {
    let mut evals = Vec::new();
    engine_0.set_board(Board::default());
    engine_1.set_board(Board::default());
    let mut result = 0.5;
    for ply in 0..160 {
        match engine_0.get_board().status() {
            cozy_chess::GameStatus::Won => {
                result = (ply % 2) as f32;
                break;
            }
            cozy_chess::GameStatus::Drawn => break,
            cozy_chess::GameStatus::Ongoing => {}
        }
        time_manager.initiate(engine_0.get_board(), time_management_info);
        let (mut make_move, eval, _, _) = engine_0.search::<Run, NoInfo>(1);
        time_manager.clear();
        let turn = match engine_0.get_board().side_to_move() {
            cozy_chess::Color::White => 1,
            cozy_chess::Color::Black => -1,
        };

        let board = engine_0.get_board().clone();
        let capture_exists = board.generate_moves(|piece_moves| {
            piece_moves.to & board.colors(!board.side_to_move()) != BitBoard::EMPTY
        });
        if capture_exists {
            evals.push((engine_0.get_board().clone(), eval * turn));
        }

        if ply < 8 {
            let mut moves = ArrayVec::<Move, 218>::new();
            board.generate_moves(|piece_moves| {
                for make_move in piece_moves {
                    moves.push(make_move);
                }
                false
            });
            make_move = moves[rand::thread_rng().gen_range(0..moves.len())];
        }
        engine_0.make_move(make_move);
        engine_1.make_move(make_move);
        if engine_0.get_position().forced_draw(0) {
            result = 0.5;
            break;
        }
        std::mem::swap(engine_0, engine_1);
    }
    evals
        .into_iter()
        .map(|(b, e)| (b, e, result))
        .collect::<Vec<_>>()
}

fn gen_games(iter: usize) -> Vec<(Board, Evaluation, f32)> {
    let mut evals = vec![];
    let time_management_options = TimeManagementInfo::MaxDepth(6);
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
        for (board, eval, wdl) in evals {
            output += &format!("{} | {} | {}\n", &board.to_string(), eval.raw(), wdl);
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
