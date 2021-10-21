use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use chess::{Board, ChessMove, Color};

use crate::bm::bm_runner::ab_runner::AbRunner;
use crate::bm::bm_runner::config::{Run, UciInfo};

use crate::bm::bm_runner::time::{TimeManagementInfo, TimeManager};

const VERSION: &str = "dev";

pub struct UciAdapter {
    bm_runner: Arc<Mutex<AbRunner>>,
    time_manager: Arc<TimeManager>,
    analysis: Option<JoinHandle<()>>,
    forced: bool,
    threads: u8,
}

impl UciAdapter {
    pub fn new() -> Self {
        let time_manager = Arc::new(TimeManager::new());
        let bm_runner = Arc::new(Mutex::new(AbRunner::new(
            Board::default(),
            time_manager.clone(),
        )));
        Self {
            bm_runner,
            threads: 1,
            forced: false,
            analysis: None,
            time_manager,
        }
    }

    pub fn input(&mut self, input: String) -> bool {
        let command = UciCommand::new(&input);
        match command {
            UciCommand::Uci => {
                println!("id name Black Marlin {}", VERSION);
                println!("id author Doruk S.");
                println!("option name Hash type spin default 16 min 1 max 65536");
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
                self.exit();
            }
            UciCommand::Quit => {
                return false;
            }
            UciCommand::Eval => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                println!("{}", runner.raw_eval().raw());
            }
            UciCommand::Go(commands) => self.go(commands),
            UciCommand::NewGame => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.set_board(Board::default());
            }
            UciCommand::Position(position, moves) => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.set_board_no_reset(position);
                for make_move in moves {
                    runner.make_move_no_reset(make_move);
                }
            }
            UciCommand::SetOption(name, value) => {
                let name: &str = &name;
                self.time_manager.abort_now();
                match name {
                    "Hash" => {
                        self.bm_runner.lock().unwrap().hash(value.parse::<usize>().unwrap());
                    }   
                    _ => {}
                }
            }
        }
        true
    }

    fn go(&mut self, commands: Vec<TimeManagementInfo>) {
        self.exit();
        self.forced = false;
        self.time_manager
            .initiate(self.bm_runner.lock().unwrap().get_board(), &commands);
        let bm_runner = self.bm_runner.clone();
        let threads = self.threads;
        self.analysis = Some(std::thread::spawn(move || {
            let (best_move, _, _ , _) = bm_runner.lock().unwrap().search::<Run, UciInfo>(threads);
            println!("bestmove {}", best_move);
        }));
    }

    fn exit(&mut self) {
        if let Some(analysis) = self.analysis.take() {
            analysis.join().unwrap();
        }
    }
}

enum UciCommand {
    Uci,
    IsReady,
    NewGame,
    Position(Board, Vec<ChessMove>),
    Go(Vec<TimeManagementInfo>),
    SetOption(String, String),
    Move(ChessMove),
    Empty,
    Stop,
    Quit,
    Eval,
}

impl UciCommand {
    fn new(input: &str) -> Self {
        let input_move = chess::ChessMove::from_str(input);
        if let Ok(m) = input_move {
            return UciCommand::Move(m);
        }
        let mut split = input.split_ascii_whitespace();
        let token = match split.next() {
            None => {
                return UciCommand::Empty;
            }
            Some(string) => string,
        };
        match token {
            "uci" => UciCommand::Uci,
            "ucinewgame" => UciCommand::NewGame,
            "position" => {
                let mut board = "".to_string();
                let mut chess_board = None;
                let split = split.into_iter().collect::<Vec<_>>();

                let mut board_end = 0;
                for (index, token) in split.iter().enumerate() {
                    let token = token.trim();
                    if token == "startpos" {
                        chess_board = Some(Board::default());
                        board_end = index + 1;
                        break;
                    } else if token != "fen" {
                        if token == "moves" {
                            if let Ok(board) = Board::from_str(&board) {
                                chess_board = Some(board);
                                board_end = index;
                                break;
                            }
                        }
                        board += token;
                        board += " ";
                    }
                }
                if chess_board.is_none() {
                    chess_board = Some(Board::from_str(&board).unwrap());
                }
                let mut moves = vec![];
                if board_end < split.len() {
                    if split[board_end] == "moves" {
                        for token in &split[board_end + 1..] {
                            moves.push(ChessMove::from_str(token).unwrap());
                        }
                    }
                }
                UciCommand::Position(chess_board.unwrap(), moves)
            }
            "go" => {
                let mut commands = vec![];
                while let Some(option) = split.next() {
                    commands.push(match option {
                        "wtime" => {
                            let millis = split.next().unwrap().parse::<i64>().unwrap();
                            let millis = millis.max(0) as u64;
                            TimeManagementInfo::WTime(Duration::from_millis(millis))
                        }
                        "btime" => {
                            let millis = split.next().unwrap().parse::<i64>().unwrap();
                            let millis = millis.max(0) as u64;
                            TimeManagementInfo::BTime(Duration::from_millis(millis))
                        }
                        "winc" => {
                            let millis = split.next().unwrap().parse::<u64>().unwrap();
                            TimeManagementInfo::WInc(Duration::from_millis(millis))
                        }
                        "binc" => {
                            let millis = split.next().unwrap().parse::<u64>().unwrap();
                            TimeManagementInfo::BInc(Duration::from_millis(millis))
                        }
                        "movetime" => {
                            let millis = split.next().unwrap().parse::<u64>().unwrap();
                            TimeManagementInfo::MoveTime(Duration::from_millis(millis))
                        }
                        "movestogo" => {
                            let moves_to_go = split.next().unwrap().parse::<u32>().unwrap();
                            TimeManagementInfo::MovesToGo(moves_to_go)
                        }
                        "depth" => {
                            let depth = split.next().unwrap().parse::<u32>().unwrap();
                            TimeManagementInfo::MaxDepth(depth)
                        }
                        "nodes" => {
                            let nodes = split.next().unwrap().parse::<u32>().unwrap();
                            TimeManagementInfo::MaxNodes(nodes)
                        }
                        _ => TimeManagementInfo::Unknown,
                    });
                }
                UciCommand::Go(commands)
            }
            "stop" => UciCommand::Stop,
            "quit" => UciCommand::Quit,
            "eval" => UciCommand::Eval,
            "isready" => UciCommand::IsReady,
            "setoption" => {
                split.next();
                let name = split.next().unwrap().to_string();
                split.next();
                let value = split.next().unwrap().to_string();
                UciCommand::SetOption(name, value)
            }
            _ => UciCommand::Empty,
        }
    }
}
