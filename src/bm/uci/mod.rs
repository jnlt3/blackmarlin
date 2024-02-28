use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cozy_chess::{Board, File, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::AbRunner;
use crate::bm::bm_runner::config::{NoInfo, Run, UciInfo};

use crate::bm::bm_runner::time::{TimeManagementInfo, TimeManager};

mod bench;
mod command;

use command::UciCommand;

const VERSION: &str = "9.0";

enum ThreadReq {
    Go(GoReq),
    Quit,
}

struct GoReq {
    bm_runner: Arc<Mutex<AbRunner>>,
    chess960: bool,
}

pub struct UciAdapter {
    bm_runner: Arc<Mutex<AbRunner>>,
    time_manager: Arc<TimeManager>,

    sender: Sender<ThreadReq>,
    forced: bool,
    chess960: bool,
    show_wdl: bool,
}

impl UciAdapter {
    pub fn new() -> Self {
        let time_manager = Arc::new(TimeManager::new());
        let bm_runner = Arc::new(Mutex::new(AbRunner::new(
            Board::default(),
            time_manager.clone(),
        )));

        let (tx, rx): (Sender<ThreadReq>, Receiver<ThreadReq>) = mpsc::channel();
        std::thread::spawn(move || loop {
            if let Ok(req) = rx.recv() {
                match req {
                    ThreadReq::Go(req) => {
                        let mut bm_runner = req.bm_runner.lock().unwrap();
                        let (mut best_move, _, _, _) = bm_runner.search::<Run, UciInfo>();
                        convert_move_to_uci(&mut best_move, bm_runner.get_board(), req.chess960);
                        println!("bestmove {}", best_move);
                    }
                    ThreadReq::Quit => {
                        return;
                    }
                }
            }
        });
        Self {
            bm_runner,
            forced: false,
            sender: tx,
            time_manager,
            chess960: false,
            show_wdl: false,
        }
    }

    pub fn input(&mut self, input: &str) -> bool {
        let name = "Black Marlin".to_string();
        let command = UciCommand::parse(&input, self.chess960);
        match command {
            UciCommand::Uci => {
                println!("id name {} {}", name, VERSION);
                println!("id author Doruk S.");
                println!("option name Hash type spin default 16 min 1 max 65536");
                println!("option name Threads type spin default 1 min 1 max 255");
                println!("option name UCI_ShowWDL type check default false");
                println!("option name UCI_Chess960 type check default false");
                println!("uciok");
            }
            UciCommand::IsReady => println!("readyok"),
            UciCommand::Move(make_move) => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.make_move(make_move);
            }
            UciCommand::Empty => {}
            UciCommand::Stop => {
                self.time_manager.abort_now();
            }
            UciCommand::Quit => {
                self.exit();
                return false;
            }
            UciCommand::Eval => {
                let runner = &mut *self.bm_runner.lock().unwrap();

                println!("eval    : {}", runner.raw_eval().raw());
            }
            UciCommand::Go(commands) => self.go(commands),
            UciCommand::NewGame => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.new_game();
                runner.set_board(Board::default());
            }
            UciCommand::Position(position, moves) => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.set_board(position);
                for mut make_move in moves {
                    convert_move(&mut make_move, runner.get_board(), self.chess960);
                    runner.make_move(make_move);
                }
            }
            UciCommand::SetOption(name, value) => {
                self.time_manager.abort_now();
                match name.as_str() {
                    "Hash" => {
                        self.bm_runner.lock().unwrap().hash(value.parse().unwrap());
                    }
                    "Threads" => {
                        self.bm_runner
                            .lock()
                            .unwrap()
                            .set_threads(value.parse().unwrap());
                    }
                    "UCI_Chess960" => {
                        self.chess960 = value.to_lowercase().parse().unwrap();
                        self.bm_runner.lock().unwrap().set_chess960(self.chess960);
                    }
                    "UCI_ShowWDL" => {
                        self.show_wdl = value.to_lowercase().parse().unwrap();
                        self.bm_runner
                            .lock()
                            .unwrap()
                            .set_uci_show_wdl(self.show_wdl);
                    }
                    _ => {}
                }
            }
            UciCommand::Bench(depth) => {
                let mut bench_data = vec![];

                let bm_runner = &mut *self.bm_runner.lock().unwrap();
                let mut sum_node_cnt = 0;
                let mut sum_time = Duration::from_nanos(0);
                for board in bench::bench_positions() {
                    bm_runner.new_game();
                    bm_runner.set_board(board.clone());
                    let options = [TimeManagementInfo::MaxDepth(depth)];
                    let start = Instant::now();

                    self.time_manager.initiate(&board, &options);
                    let (make_move, eval, _, node_cnt) = bm_runner.search::<Run, NoInfo>();
                    self.time_manager.clear();
                    let elapsed = start.elapsed();
                    bench_data.push((
                        eval.raw(),
                        make_move,
                        node_cnt,
                        (node_cnt as f32 / elapsed.as_secs_f32()) as u32,
                    ));
                    sum_time += elapsed;
                    sum_node_cnt += node_cnt;
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
                    sum_node_cnt,
                    (sum_node_cnt as f32 / sum_time.as_secs_f32()) as u32
                );
            }
            UciCommand::Static => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                println!("{}", runner.raw_eval().raw());
            }
        }
        true
    }

    fn go(&mut self, commands: Vec<TimeManagementInfo>) {
        self.forced = false;
        self.time_manager
            .initiate(self.bm_runner.lock().unwrap().get_board(), &commands);
        let bm_runner = self.bm_runner.clone();
        let chess960 = self.chess960;

        let req = GoReq {
            bm_runner,
            chess960,
        };
        self.sender.send(ThreadReq::Go(req)).unwrap();
    }

    fn exit(&mut self) {
        self.time_manager.abort_now();
        self.sender.send(ThreadReq::Quit).unwrap();
    }
}

pub fn convert_move_to_uci(make_move: &mut Move, board: &Board, chess960: bool) {
    if !chess960 && board.color_on(make_move.from) == board.color_on(make_move.to) {
        let rights = board.castle_rights(board.side_to_move());
        let file = if Some(make_move.to.file()) == rights.short {
            File::G
        } else {
            File::C
        };
        make_move.to = Square::new(file, make_move.to.rank());
    }
}

fn convert_move(make_move: &mut Move, board: &Board, chess960: bool) {
    let convert_castle = !chess960
        && board.piece_on(make_move.from) == Some(Piece::King)
        && make_move.from.file() == File::E
        && matches!(make_move.to.file(), File::C | File::G);
    if convert_castle {
        let file = if make_move.to.file() == File::C {
            File::A
        } else {
            File::H
        };
        make_move.to = Square::new(file, make_move.to.rank());
    }
}
