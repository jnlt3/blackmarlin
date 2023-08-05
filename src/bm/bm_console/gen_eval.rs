use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::{mpsc::channel, Arc},
    time::{Duration, Instant},
};

use arrayvec::ArrayVec;
use cozy_chess::{BitBoard, Board, Move};
use rand::Rng;

use crate::bm::{
    bm_runner::{
        ab_runner::AbRunner,
        config::{NoInfo, Run},
        time::{TimeManagementInfo, TimeManager},
    },
    bm_util::eval::Evaluation,
};

use threadpool::{self, ThreadPool};

fn play_single(
    engine: &mut AbRunner,
    time_manager: &TimeManager,
    time_management_info: &[TimeManagementInfo],
) -> Vec<(Board, Evaluation, f32)> {
    let mut evals = Vec::new();
    engine.set_board(Board::default());
    let mut result = 0.5;
    for ply in 0.. {
        match engine.get_board().status() {
            cozy_chess::GameStatus::Won => {
                result = (ply % 2) as f32;
                break;
            }
            cozy_chess::GameStatus::Drawn => break,
            cozy_chess::GameStatus::Ongoing => {}
        }
        time_manager.initiate(engine.get_board(), time_management_info);
        let (mut make_move, eval, _, _) = engine.search::<Run, NoInfo>();
        time_manager.clear();
        let turn = match engine.get_board().side_to_move() {
            cozy_chess::Color::White => 1,
            cozy_chess::Color::Black => -1,
        };

        let board = engine.get_board().clone();

        if ply > 16
            && !board
                .colors(!engine.get_board().side_to_move())
                .has(make_move.to)
            && board.checkers() == BitBoard::EMPTY
        {
            evals.push((engine.get_board().clone(), eval * turn));
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
        engine.make_move(make_move);
        if engine.get_position().forced_draw(0) {
            result = 0.5;
            break;
        }
    }
    evals
        .into_iter()
        .map(|(b, e)| (b, e, result))
        .collect::<Vec<_>>()
}

fn gen_games(duration: Duration, depth: u32) -> Vec<(Board, Evaluation, f32)> {
    let start = Instant::now();
    let mut evals = vec![];
    let time_management_options = TimeManagementInfo::MaxDepth(depth);
    let time_manager = Arc::new(TimeManager::new());
    let mut engine_0 = AbRunner::new(Board::default(), time_manager.clone());
    while start.elapsed() < duration {
        evals.extend(play_single(
            &mut engine_0,
            &time_manager,
            &[time_management_options],
        ));
        engine_0.new_game();
    }
    evals
}

pub fn gen_eval(depth: u32, thread_cnt: u32, target_path: &str) {
    let pool = ThreadPool::new(thread_cnt as usize);
    loop {
        let (tx, rx) = channel();
        for _ in 0..thread_cnt {
            let tx = tx.clone();
            pool.execute(move || {
                tx.send(gen_games(Duration::from_secs(30), depth)).unwrap();
            });
        }
        let mut output = String::new();
        for (board, eval, wdl) in rx.iter().take(thread_cnt as usize).flatten() {
            output += &format!("{} | {} | {}\n", &board.to_string(), eval.raw(), wdl);
        }
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(target_path)
            .unwrap();
        let mut write = BufWriter::new(file);
        write.write(output.as_bytes()).unwrap();
    }
}
