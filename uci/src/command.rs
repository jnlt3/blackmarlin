use std::time::Duration;

use cozy_chess::{Board, Move};

use blackmarlin::bm::bm_runner::time::TimeManagementInfo;

pub enum UciCommand {
    Uci,
    IsReady,
    NewGame,
    Position(Board, Vec<Move>),
    Go(Vec<TimeManagementInfo>),
    SetOption(String, String),
    Move(Move),
    Bench(u32),
    Empty,
    Stop,
    Quit,
    Eval,
    Static,
}

impl UciCommand {
    pub fn parse(input: &str, chess960: bool) -> Self {
        let input_move = input.parse();
        if let Ok(m) = input_move {
            return UciCommand::Move(m);
        }
        let mut split = input.split_ascii_whitespace();
        let token = match split.next() {
            Some(string) => string,
            None => return UciCommand::Empty,
        };
        match token {
            "uci" => UciCommand::Uci,
            "ucinewgame" => UciCommand::NewGame,
            "position" => {
                let mut board = String::new();
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
                            if let Ok(board) = Board::from_fen(board.trim(), chess960) {
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
                    chess_board = Some(Board::from_fen(board.trim(), chess960).unwrap());
                }
                let mut moves = vec![];
                if board_end < split.len() && split[board_end] == "moves" {
                    for token in &split[board_end + 1..] {
                        let make_move = token.parse().unwrap();
                        moves.push(make_move);
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
                            let millis = split.next().unwrap().parse().unwrap();
                            TimeManagementInfo::WInc(Duration::from_millis(millis))
                        }
                        "binc" => {
                            let millis = split.next().unwrap().parse().unwrap();
                            TimeManagementInfo::BInc(Duration::from_millis(millis))
                        }
                        "movetime" => {
                            let millis = split.next().unwrap().parse().unwrap();
                            TimeManagementInfo::MoveTime(Duration::from_millis(millis))
                        }
                        "movestogo" => {
                            let moves_to_go = split.next().unwrap().parse().unwrap();
                            TimeManagementInfo::MovesToGo(moves_to_go)
                        }
                        "depth" => {
                            let depth = split.next().unwrap().parse().unwrap();
                            TimeManagementInfo::MaxDepth(depth)
                        }
                        "nodes" => {
                            let nodes = split.next().unwrap().parse().unwrap();
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
            "bench" => UciCommand::Bench(split.next().map_or(12, |depth| depth.parse().unwrap())),
            "static" => UciCommand::Static,
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
