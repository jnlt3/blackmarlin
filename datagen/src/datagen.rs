use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, mpsc::channel},
    time::{Duration, Instant},
};

use arrayvec::ArrayVec;
use cozy_chess::{Board, Move};
use rand::RngExt;

use blackmarlin::bm::{
    bm_runner::{
        searcher::{self, SearchCtx, SearchObserver, SharedCtx, ThreadData},
        time::TimeManagementInfo,
    },
    bm_util::{eval::Evaluation, position::Position},
};

use threadpool::{self, ThreadPool};

struct EngineState {
    search_ctx: SearchCtx,
    shared_ctx: SharedCtx,
    thread_data: ThreadData,
}

struct NullObserver;

impl SearchObserver for NullObserver {}

fn play_single(
    engine: &mut EngineState,
    time_management_info: &[TimeManagementInfo],
    random_plies: usize,
    variant: u8,
    draw_adj: bool,
) -> Vec<(Board, Evaluation, f32, Move, usize)> {
    let mut evals = Vec::new();

    let start_board = match variant {
        0 => Board::default(),
        1 => Board::chess960_startpos(rand::rng().random_range(0..960)),
        2 => Board::double_chess960_startpos(
            rand::rng().random_range(0..960),
            rand::rng().random_range(0..960),
        ),
        _ => panic!("Invalid variant"),
    };
    engine.search_ctx.root.set_board(start_board);
    let mut result = 0.5;
    let mut draw_cnt = 0;
    for ply in 0.. {
        match engine.search_ctx.root.board().status() {
            cozy_chess::GameStatus::Won => {
                result = (ply % 2) as f32;
                break;
            }
            cozy_chess::GameStatus::Drawn => break,
            cozy_chess::GameStatus::Ongoing => {}
        }

        let make_move = if ply < random_plies {
            let mut moves = ArrayVec::<Move, 218>::new();
            engine
                .search_ctx
                .root
                .board()
                .generate_moves(|piece_moves| {
                    for make_move in piece_moves {
                        moves.push(make_move);
                    }
                    false
                });
            moves[rand::rng().random_range(0..moves.len())]
        } else {
            // Random moves and no eval are simply not useful data
            engine
                .shared_ctx
                .time_manager
                .initiate(engine.search_ctx.root.board(), time_management_info);
            engine.search_ctx.root.reset();
            let (mv, eval) = searcher::root(
                &engine.shared_ctx,
                &mut engine.thread_data,
                &mut engine.search_ctx,
                &NullObserver,
            )
            .unwrap();
            engine.shared_ctx.time_manager.clear();
            let turn = match engine.search_ctx.root.board().side_to_move() {
                cozy_chess::Color::White => 1,
                cozy_chess::Color::Black => -1,
            };
            if eval.raw() == 0 && ply >= 80 {
                draw_cnt += 1;
            } else {
                draw_cnt = 0;
            }
            evals.push((engine.search_ctx.root.board().clone(), eval * turn, mv, ply));
            mv
        };
        engine.search_ctx.root.make_move(make_move);
        if engine.search_ctx.root.forced_draw(1) || (draw_adj && draw_cnt >= 8) {
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
    draw_adj: bool,
) -> Vec<(Board, Evaluation, f32, Move, usize)> {
    let start = Instant::now();
    let mut evals = vec![];
    let mut engine_0 = EngineState {
        search_ctx: SearchCtx {
            root: Position::new(Board::default()),
            start,
        },
        shared_ctx: SharedCtx::new_with_hash(1),
        thread_data: ThreadData::new_with_id(0),
    };
    while start.elapsed() < duration {
        evals.extend(play_single(
            &mut engine_0,
            tm_options,
            random_plies,
            variant,
            draw_adj,
        ));
        engine_0.shared_ctx.t_table.clear();
        engine_0.thread_data.clear();
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
    pub draw_adj: bool,
}

pub fn gen_eval(tm_options: &Arc<[TimeManagementInfo]>, options: DataGenOptions) {
    let mut fen_count = 0;
    let pool = ThreadPool::new(options.threads);
    loop {
        let start_fen_count = fen_count;
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
                    options.draw_adj,
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
        let positions_per_second = (fen_count - start_fen_count) as u64 / options.interval;
        eprintln!("{:.2} pos/s", positions_per_second);
        if fen_count >= options.pos_count {
            break;
        }
    }
}
