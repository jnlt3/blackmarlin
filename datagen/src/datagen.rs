use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{mpsc::channel, Arc},
    time::{Duration, Instant},
};

use arrayvec::ArrayVec;
use cozy_chess::{Board, Move};
use rand::Rng;

use blackmarlin::bm::{
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
    random_plies: usize,
    variant: u8,
) -> Vec<(Board, Evaluation, f32, Move, usize)> {
    let mut evals = Vec::new();

    let start_board = match variant {
        0 => Board::default(),
        1 => Board::chess960_startpos(rand::thread_rng().gen_range(0..960)),
        2 => Board::double_chess960_startpos(
            rand::thread_rng().gen_range(0..960),
            rand::thread_rng().gen_range(0..960),
        ),
        _ => panic!("Invalid variant"),
    };
    engine.set_board(start_board);
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

        let make_move = if ply < random_plies {
            let mut moves = ArrayVec::<Move, 218>::new();
            engine.get_board().generate_moves(|piece_moves| {
                for make_move in piece_moves {
                    moves.push(make_move);
                }
                false
            });
            moves[rand::thread_rng().gen_range(0..moves.len())]
        } else {
            // Random moves and no eval are simply not useful data
            time_manager.initiate(engine.get_board(), time_management_info);
            let (mv, eval, _, _) = engine.search::<Run, NoInfo>();
            time_manager.clear();
            let turn = match engine.get_board().side_to_move() {
                cozy_chess::Color::White => 1,
                cozy_chess::Color::Black => -1,
            };
            evals.push((engine.get_board().clone(), eval * turn, mv, ply));
            mv
        };
        engine.make_move(make_move);
        if engine.get_position().forced_draw(1) {
            result = 0.5;
            break;
        }
    }
    evals
        .into_iter()
        .map(|(b, e, mv, ply)| (b, e, result, mv, ply))
        .collect::<Vec<_>>()
}

fn gen_games(
    duration: Duration,
    tm_options: &[TimeManagementInfo],
    random_plies: usize,
    variant: u8,
) -> Vec<(Board, Evaluation, f32, Move, usize)> {
    let start = Instant::now();
    let mut evals = vec![];
    let time_manager = Arc::new(TimeManager::new());
    let mut engine_0 = AbRunner::new(Board::default(), time_manager.clone());
    while start.elapsed() < duration {
        evals.extend(play_single(
            &mut engine_0,
            &time_manager,
            tm_options,
            random_plies,
            variant,
        ));
        engine_0.new_game();
    }
    evals
}

pub struct DataGenOptions {
    pub threads: usize,
    pub random_plies: usize,
    pub pos_count: usize,
    pub variant: u8,
    pub out: PathBuf,
    pub interval: u64,
}

pub fn gen_eval(tm_options: &Arc<[TimeManagementInfo]>, options: DataGenOptions) {
    let mut fen_count = 0;
    let pool = ThreadPool::new(options.threads);
    loop {
        let (tx, rx) = channel();
        for _ in 0..options.threads {
            let tx = tx.clone();
            let tm_options = tm_options.clone();
            pool.execute(move || {
                tx.send(gen_games(
                    Duration::from_secs(options.interval),
                    &tm_options,
                    options.random_plies,
                    options.variant,
                ))
                .unwrap();
            });
        }
        let mut output = String::new();
        for (board, eval, wdl, mv, ply) in rx.iter().take(options.threads).flatten() {
            output += &format!(
                "{} | {} | {} | {} | {}\n",
                &board.to_string(),
                eval.raw(),
                wdl,
                mv,
                ply
            );
            fen_count += 1;
        }
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&options.out)
            .unwrap();
        let mut write = BufWriter::new(file);
        write.write(output.as_bytes()).unwrap();
        if fen_count >= options.pos_count {
            break;
        }
    }
}
