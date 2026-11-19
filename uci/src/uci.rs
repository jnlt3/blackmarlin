use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use blackmarlin::bm::bm_runner::searcher::{SearchObserver, Searcher};
use blackmarlin::bm::bm_util::eval::Evaluation;
use blackmarlin::bm::bm_util::position::Position;
use cozy_chess::{Board, File, Move, Piece, Square};

use blackmarlin::bm::bm_runner::time::TimeManagementInfo;

use crate::bench;
use crate::command::UciCommand;

const VERSION: &str = "9.0";

const DEFAULT_HASH_MIB: usize = 16;
const DEFAULT_THREADS: usize = 1;

pub struct UciAdapter {
    searcher: Searcher,

    chess960: bool,
    show_wdl: bool,

    root: Position,
}

struct BmUciStandard {
    frc: bool,
    wdl: bool,
    thread_nodes: Box<[AtomicU64]>,
}

impl BmUciStandard {
    fn remove_aggression(eval: Evaluation, scale: i32) -> Evaluation {
        const MAX: i32 = 200;
        match eval.is_mate() {
            true => eval,
            false => {
                let eval = eval.raw() as i32;
                let eval = eval - scale * eval.clamp(-MAX - scale, MAX + scale) / (100 + scale);
                Evaluation::new(eval as i16)
            }
        }
    }

    pub fn convert_move_to_uci(&self, mut mv: Move, board: &Board) -> Move {
        if !self.frc && board.color_on(mv.from) == board.color_on(mv.to) {
            let rights = board.castle_rights(board.side_to_move());
            let file = if Some(mv.to.file()) == rights.short {
                File::G
            } else {
                File::C
            };
            mv.to = Square::new(file, mv.to.rank());
        }
        mv
    }

    fn to_wld(eval: Evaluation) -> (i16, i16, i16) {
        if let Some(mate_in) = eval.mate_in() {
            return match mate_in {
                _ if mate_in > 0 => (1000, 0, 0),
                _ if mate_in < 0 => (0, 0, 1000),
                _ => unreachable!(),
            };
        }
        let raw = eval.raw().clamp(-2000, 2000) as f32 * 0.01;
        let mut wdl = [raw * 0.964, raw * -0.964, 2.68];
        wdl.iter_mut().for_each(|x| *x = x.exp());
        let sum: f32 = wdl.iter().sum();
        wdl.iter_mut().for_each(|x| *x *= 1000.0 / sum);

        (
            wdl[0] as i16,
            wdl[1] as i16,
            1000 - (wdl[0] as i16 + wdl[1] as i16),
        )
    }
}

fn convert_move(frc: bool, mut mv: Move, board: &Board) -> Move {
    let convert_castle = !frc
        && board.piece_on(mv.from) == Some(Piece::King)
        && mv.from.file() == File::E
        && matches!(mv.to.file(), File::C | File::G);
    if convert_castle {
        let file = if mv.to.file() == File::C {
            File::A
        } else {
            File::H
        };
        mv.to = Square::new(file, mv.to.rank());
    }
    mv
}
impl SearchObserver for BmUciStandard {
    fn search_result(
        &self,
        thread_id: usize,
        root: &Board,
        eval: Evaluation,
        elapsed: Duration,
        depth: u32,
        sel_depth: u32,
        nodes: u64,
        pv: &[Option<cozy_chess::Move>],
    ) {
        self.thread_nodes[thread_id].store(nodes, Ordering::Relaxed);
        if thread_id != 0 {
            return;
        }
        let eval_str = if eval.is_mate() {
            format!("mate {}", eval.mate_in().unwrap())
        } else {
            format!("cp {}", eval.raw())
        };
        let nodes: u128 = self
            .thread_nodes
            .iter()
            .map(|nodes| nodes.load(Ordering::Relaxed) as u128)
            .sum();
        let nps = (nodes * 1000) / elapsed.as_millis().max(1);

        let mut board = root.clone();

        let mut pv_buffer = String::new();
        for make_move in pv {
            let Some(mv) = *make_move else {
                break;
            };

            let uci_move = self.convert_move_to_uci(mv, &board);
            board.play_unchecked(mv);
            write!(&mut pv_buffer, " {}", uci_move).unwrap();
        }

        let scale = board.occupied().len() - board.pieces(Piece::Pawn).len();
        let eval = Self::remove_aggression(eval, scale as i32 * 2);
        let wld = match self.wdl {
            true => Some(Self::to_wld(eval)),
            false => None,
        };
        let wdl = match wld {
            Some(wld) => format!(" wdl {} {} {}", wld.0, wld.2, wld.1),
            None => "".to_string(),
        };
        let output = format!(
            "info depth {} seldepth {} score {}{} time {} nodes {} nps {} pv{}",
            depth,
            sel_depth,
            eval_str,
            wdl,
            elapsed.as_millis(),
            nodes,
            nps,
            pv_buffer,
        );
        println!("{}", output);
    }

    fn result(&self, best_move: Move, _eval: Evaluation) {
        println!("bestmove {best_move}")
    }
}

#[derive(Default)]
struct BmBench {
    nodes: AtomicU64,
    result: Mutex<Option<(Move, Evaluation)>>,
}

impl SearchObserver for BmBench {
    fn search_result(
        &self,
        _thread_id: usize,
        _root: &Board,
        _eval: Evaluation,
        _elapsed: Duration,
        _depth: u32,
        _sel_depth: u32,
        nodes: u64,
        _pv: &[Option<Move>],
    ) {
        self.nodes.store(nodes, Ordering::Relaxed);
    }

    fn result(&self, best_move: Move, eval: Evaluation) {
        *self.result.lock().unwrap() = Some((best_move, eval));
    }
}

impl BmBench {
    fn result(&self) -> Option<(u64, Move, Evaluation)> {
        let Some((mv, eval)) = *self.result.lock().unwrap() else {
            return None;
        };
        Some((self.nodes.load(Ordering::Relaxed), mv, eval))
    }
}

impl UciAdapter {
    pub fn new() -> Self {
        Self {
            searcher: Searcher::new(DEFAULT_HASH_MIB, DEFAULT_THREADS),
            chess960: false,
            show_wdl: false,
            root: Position::new(Board::default()),
        }
    }

    pub fn input(&mut self, input: &str) -> bool {
        let name = "Black Marlin".to_string();
        let command = UciCommand::parse(&input, self.chess960);
        match command {
            UciCommand::Uci => {
                println!("id name {} {}", name, VERSION);
                println!("id author jnlt3");
                println!("option name Hash type spin default 16 min 1 max 65536");
                println!("option name Threads type spin default 1 min 1 max 65535");
                println!("option name UCI_ShowWDL type check default false");
                println!("option name UCI_Chess960 type check default false");
                println!("uciok");
            }
            UciCommand::IsReady => println!("readyok"),
            UciCommand::Go(commands) => self.go(commands),
            UciCommand::Stop => {
                self.searcher.context.time_manager.abort_now();
                self.searcher.wait();
            }
            UciCommand::Move(make_move) => {
                self.root.make_move(make_move);
            }
            UciCommand::Empty => {}
            UciCommand::NewGame => {
                self.searcher.clear();
            }
            UciCommand::SetOption(name, value) => match name.as_str() {
                "Hash" => {
                    self.searcher.set_hash(value.parse().unwrap());
                }
                "Threads" => {
                    self.searcher.set_threads(value.parse().unwrap());
                }
                "UCI_Chess960" => {
                    self.chess960 = value.to_lowercase().parse().unwrap();
                }
                "UCI_ShowWDL" => {
                    self.show_wdl = value.to_lowercase().parse().unwrap();
                }
                _ => {}
            },
            UciCommand::Quit => {
                self.exit();
                return false;
            }
            UciCommand::Position(position, moves) => {
                self.root.set_board(position);
                for mv in moves {
                    let mv = convert_move(self.chess960, mv, self.root.board());
                    self.root.make_move(mv);
                }
                self.root.reset();
            }
            UciCommand::Bench(depth) => {
                let mut bench_data = vec![];

                let mut total_nodes = 0;
                let mut sum_time = Duration::from_nanos(0);
                for board in bench::bench_positions() {
                    self.root.set_board(board);
                    self.searcher.clear();
                    self.searcher
                        .context
                        .time_manager
                        .initiate(&self.root.board(), &[TimeManagementInfo::MaxDepth(depth)]);

                    let bench_observer = Arc::new(BmBench::default());

                    let start = Instant::now();
                    self.searcher
                        .search(self.root.clone(), bench_observer.clone());
                    self.searcher.wait();
                    self.searcher.context.time_manager.clear();
                    let elapsed = start.elapsed();
                    let (nodes, mv, eval) = bench_observer.result().unwrap();
                    bench_data.push((
                        eval.raw(),
                        mv,
                        nodes,
                        (nodes as f32 / elapsed.as_secs_f32()) as u32,
                    ));
                    sum_time += elapsed;
                    total_nodes += nodes;
                }
                let mut divider_size = 0;
                for (index, (cp, mv, nodes, nps)) in bench_data.into_iter().enumerate() {
                    let line = &format!(
                        "[#{:>3}]{:>8} cp  Best: {:>8} {:>8} nodes {:>8} nps",
                        index + 1,
                        cp,
                        mv,
                        nodes,
                        nps
                    );
                    divider_size = divider_size.max(line.chars().count());
                    println!("{}", line);
                }
                println!("{}", "=".repeat(divider_size));
                println!(
                    "OVERALL {:>30} nodes {:>8} nps",
                    total_nodes,
                    (total_nodes as f32 / sum_time.as_secs_f32()) as u32
                );
            }
            UciCommand::Eval => {
                println!("eval: {}", self.root.get_eval().raw())
            }
        }
        true
    }

    fn go(&mut self, commands: Vec<TimeManagementInfo>) {
        self.searcher
            .context
            .time_manager
            .initiate(self.root.board(), &commands);
        self.searcher.search(
            self.root.clone(),
            Arc::new(BmUciStandard {
                frc: self.chess960,
                wdl: self.show_wdl,
                thread_nodes: (0..self.searcher.threads())
                    .map(|_| AtomicU64::new(0))
                    .collect(),
            }),
        );
    }

    fn exit(&mut self) {
        self.searcher.quit();
    }
}
