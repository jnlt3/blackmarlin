use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use chess::{Board, ChessMove, Color, MoveGen};

use crate::bm::bm_runner::ab_runner::AbRunner;
use crate::bm::bm_runner::config::{Run, UciInfo};

use crate::bm::bm_runner::time::{MainTimeManager, TimeManager};

const VERSION: &str = "dev";
pub struct UciAdapter {
    bm_runner: Arc<Mutex<AbRunner>>,

    time_manager: Arc<MainTimeManager>,

    analysis: Option<JoinHandle<()>>,

    forced: bool,

    w_time_left: f32,
    b_time_left: f32,
    time_per_move: f32,

    threads: u8,
}

impl UciAdapter {
    pub fn new() -> Self {
        let time_manager = Arc::new(MainTimeManager::new());
        let bm_runner = Arc::new(Mutex::new(AbRunner::new(
            Board::default(),
            time_manager.clone(),
        )));
        Self {
            bm_runner,
            w_time_left: 0_f32,
            b_time_left: 0_f32,
            time_per_move: 0_f32,
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
                println!("uciok");
            }
            UciCommand::IsReady => println!("readyok"),
            UciCommand::Move(make_move) => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.make_move(make_move);
            }
            UciCommand::Empty => {}
            UciCommand::Quit => {
                return false;
            }
            UciCommand::Eval => todo!(),
            UciCommand::Diagnostics => todo!(),
            UciCommand::Detail => todo!(),
            UciCommand::Go(commands) => {
                for command in commands {
                    match command {
                        GoCommand::WTime(time) => self.w_time_left = time * 0.001,
                        GoCommand::BTime(time) => self.b_time_left = time * 0.001,
                        GoCommand::MoveTime(time) => self.time_per_move = time * 0.001,
                        GoCommand::Empty => {}
                    }
                }
                self.go()
            }
            UciCommand::NewGame => {}
            UciCommand::Position(position, moves) => {
                let runner = &mut *self.bm_runner.lock().unwrap();
                runner.set_board_no_reset(position);
                for make_move in moves {
                    runner.make_move_no_reset(make_move);
                }
            }
        }
        true
    }

    fn go(&mut self) {
        self.exit();
        self.forced = false;
        let bm_runner = &mut *self.bm_runner.lock().unwrap();
        let time_left = match bm_runner.get_board().side_to_move() {
            Color::White => self.w_time_left,
            Color::Black => self.b_time_left,
        };
        self.time_manager.initiate(
            Duration::from_secs_f32(time_left),
            //FIXME: Hacky code
            bm_runner.get_board(),
        );
        let (make_move, _, _, _) = bm_runner.search::<Run, UciInfo>(self.threads);
        bm_runner.make_move(make_move);
        println!("bestmove {}", make_move);
        self.time_manager.clear();
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
    Go(Vec<GoCommand>),
    Move(ChessMove),
    Empty,
    Quit,
    Eval,
    Diagnostics,
    Detail,
}

enum GoCommand {
    WTime(f32),
    BTime(f32),
    MoveTime(f32),
    Empty,
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
                            let millis = split.next().unwrap().parse::<u32>().unwrap() as f32;
                            GoCommand::WTime(millis)
                        }
                        "btime" => {
                            let millis = split.next().unwrap().parse::<u32>().unwrap() as f32;
                            GoCommand::BTime(millis)
                        }
                        "movetime" => {
                            let millis = split.next().unwrap().parse::<u32>().unwrap() as f32;
                            GoCommand::MoveTime(millis)
                        }
                        _ => GoCommand::Empty,
                    });
                }
                UciCommand::Go(commands)
            }
            "quit" => UciCommand::Quit,
            "eval" => UciCommand::Eval,
            "diagnostics" => UciCommand::Diagnostics,
            "detail" => UciCommand::Detail,
            "isready" => UciCommand::IsReady,
            _ => UciCommand::Empty,
        }
    }
}
