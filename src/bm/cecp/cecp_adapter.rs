use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chess::{Board, ChessMove};

use crate::bm::bm_runner::config::{NoInfo, Run, XBoardInfo};

use crate::bm::bm_runner::runner::Runner;
use crate::bm::bm_runner::time::{
    CompoundTimeManager, ConstDepth, ConstTime, MainTimeManager, TimeManager,
};
use crate::bm::bm_util::evaluator::Evaluator;
use std::marker::PhantomData;

const POSITIONS: &[&str] = &[
    "Q7/5Q2/8/8/3k4/6P1/6BP/7K b - - 0 67",
    "r4rk1/p4ppp/1q2p3/2n1P3/2p5/3bRNP1/1P3PBP/R2Q2K1 b - - 0 24",
    "r1bq1rk1/pp3ppp/2nbpn2/3p4/3P4/1PN1PN2/1BP1BPPP/R2Q1RK1 b - - 2 10",
    "1r4k1/1P3p2/6pp/2Pp4/4P3/PQ1K1R2/6P1/4q3 w - - 0 51",
    "8/8/R7/4n3/4k3/6P1/6K1/8 w - - 68 164",
    "2r3k1/1b4bp/1p2p1p1/3pNp2/3P1P1q/PB1Q3P/1P4P1/4R1K1 w - - 2 36",
    "4rrk1/1b4bp/p1p1p1p1/3pN3/1P3q2/PQN3P1/2P1RP1P/3R2K1 b - - 0 24",
    "rnbq1rk1/ppp1bppp/4p3/3pP1n1/2PP3P/5PP1/PP4B1/RNBQK1NR b KQ - 0 8",
    "3r1r1k/p1p3pp/2p5/8/4K3/2N3Pb/PPP5/R1B4R b - - 0 20",
    "r4k1r/ppq2ppp/4bB2/8/2p5/4P3/P3BPPP/1R1Q1RK1 b - - 0 17",
    "r4rk1/1b1nq1pp/p7/3pNp2/1p3Q2/3B3P/PPP1N1R1/R2K4 w - - 2 21",
    "8/5p2/8/p6k/8/3N4/5PPK/8 w - - 0 49",
    "2r1rbk1/4pp1p/1Q1P1np1/2B1Nq2/P4P2/1B3P2/1PP3bP/1K1RR3 b - - 0 29",
    "6k1/p4ppp/Bpp5/4P3/P7/4QKPb/2P3N1/3r3q w - - 5 36",
    "3br1k1/pp1r1ppp/3pbn2/P2Np3/1PPpP3/3P1NP1/5PBP/3RR1K1 w - - 1 21",
    "8/1p6/p3n3/4k3/8/6PR/1rr5/3R2K1 w - - 8 54",
    "1r4k1/p4p1p/5p2/8/4P3/4K3/PPP3P1/4R3 w - - 0 34",
    "6k1/6p1/7p/7R/7P/5n2/P3K1b1/8 b - - 2 48",
    "2rr2k1/pp5p/3p4/4p3/2b1p3/P4QP1/1P4P1/3R2K1 w - - 0 28",
    "q1r4k/1bR5/rp4pB/3p4/3P2nQ/8/PP3PPP/R5K1 w - - 1 29",
    "rnbqkbnr/pppppp1p/6p1/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
    "rnbqk1nr/1p3ppp/p3p3/2bp4/4P3/5N2/PPPN1PPP/R1BQKB1R w KQkq - 0 6",
    "r2q1rk1/1p1b1p1p/p5p1/3QP3/8/5N2/PP3PPP/2KR3R b - - 0 20",
    "r3r2k/pbp1q2p/1p6/4n3/2NQ4/2P2pB1/P1P2P1P/2R2RK1 b - - 6 26",
    "8/1p2k3/4rp2/p2R3Q/2q2B2/6P1/5P1P/6K1 b - - 14 73",
];

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TimeManagerType {
    ConstDepth,
    ConstTime,
    Normal,
}

pub struct CecpAdapter<Eval: 'static + Clone + Send + Evaluator, R: Runner<Eval>> {
    eval_type: PhantomData<Eval>,
    bm_runner: R,

    current_time_manager: TimeManagerType,
    time_manager: Arc<CompoundTimeManager>,
    const_depth: Arc<ConstDepth>,
    const_time: Arc<ConstTime>,

    forced: bool,

    time_left: f32,

    threads: u8,
}

impl<Eval: 'static + Clone + Send + Evaluator, R: Runner<Eval>> CecpAdapter<Eval, R> {
    pub fn new() -> Self {
        let const_depth = Arc::new(ConstDepth::new(8));
        let const_time = Arc::new(ConstTime::new(Duration::from_secs(0)));
        let main_time = Arc::new(MainTimeManager::new());
        let managers: Vec<Arc<dyn TimeManager>> =
            vec![const_depth.clone(), const_time.clone(), main_time];
        let time_manager = Arc::new(CompoundTimeManager::new(
            managers.into_boxed_slice(),
            TimeManagerType::Normal as usize,
        ));
        let bm_runner = R::new(Board::default(), time_manager.clone());
        Self {
            eval_type: PhantomData::default(),
            bm_runner,
            time_left: 0_f32,
            threads: 1,
            forced: false,
            current_time_manager: TimeManagerType::Normal,
            const_depth,
            const_time,
            time_manager,
        }
    }

    pub fn features(&self) -> String {
        "feature ping=1 setboard=1 analyze=0 time=1 smp=1".to_string()
    }

    pub fn input(&mut self, input: String) -> bool {
        let command = CecpCommand::new(&input);
        match command {
            CecpCommand::XBoard => {
                println!("{}", self.features());
                println!("done=1");
            }
            CecpCommand::Move(make_move) => {
                self.bm_runner.make_move(make_move);
                if !self.forced {
                    self.go()
                }
            }
            CecpCommand::Ping(number) => {
                println!("pong {}", number);
            }
            CecpCommand::SetBoard(board) => {
                self.bm_runner.set_board(board);
            }
            CecpCommand::Level(_, time_left, _) => {
                self.time_left = time_left as f32;
                self.current_time_manager = TimeManagerType::Normal;
                self.time_manager.set_mode(TimeManagerType::Normal as usize);
            }
            CecpCommand::Time(time_left) => {
                self.time_left = time_left;
                self.current_time_manager = TimeManagerType::Normal;
                self.time_manager.set_mode(TimeManagerType::Normal as usize);
            }
            CecpCommand::SetTime(time) => {
                self.current_time_manager = TimeManagerType::ConstTime;
                self.time_manager
                    .set_mode(TimeManagerType::ConstTime as usize);
                self.const_time.set_duration(Duration::from_secs_f32(time));
            }
            CecpCommand::Go => {
                self.go();
            }
            CecpCommand::New => self.bm_runner.set_board(chess::Board::default()),
            CecpCommand::Eval => {
                println!("eval: {:?}", self.bm_runner.raw_eval());
            }
            CecpCommand::Cores(cores) => {
                self.threads = cores;
            }
            CecpCommand::Force => {
                self.forced = true;
            }
            CecpCommand::Quit => {
                return false;
            }
            //TODO: reset back to the old position
            CecpCommand::Bench => {
                let prev = self.current_time_manager;
                self.current_time_manager = TimeManagerType::ConstDepth;
                self.time_manager
                    .set_mode(TimeManagerType::ConstDepth as usize);
                self.const_depth.set_depth(8);
                self.bench();
                self.current_time_manager = prev;
            }
            CecpCommand::Perf => {
                let prev = self.current_time_manager;
                self.current_time_manager = TimeManagerType::ConstTime;
                self.time_manager
                    .set_mode(TimeManagerType::ConstTime as usize);
                self.const_time.set_duration(Duration::from_secs(1));
                self.bench();
                self.current_time_manager = prev;
            }
            CecpCommand::Empty => {}
            CecpCommand::MoveNow => {
                //TODO:
            }
        }
        true
    }

    fn go(&mut self) {
        self.forced = false;
        self.time_manager
            .initiate(Duration::from_secs_f32(self.time_left));
        let (make_move, _, _, _) = self
            .bm_runner
            .search::<Run, XBoardInfo>(self.threads, false);
        self.bm_runner.make_move(make_move);
        println!("move {}", make_move);
        self.time_manager.clear();
    }

    fn bench(&mut self) {
        let mut sum_node_cnt = 0;
        let mut sum_time = Duration::from_nanos(0);
        for position in POSITIONS {
            self.bm_runner
                .set_board(chess::Board::from_str(position).unwrap());

            let start = Instant::now();
            let (_, _, _, node_cnt) = self.bm_runner.search::<Run, NoInfo>(1, false);
            sum_time += start.elapsed();
            self.const_depth.clear();
            sum_node_cnt += node_cnt;
        }
        println!(
            "nps: {}, node_cnt: {}",
            sum_node_cnt as f32 / sum_time.as_secs_f32(),
            sum_node_cnt,
        )
    }
}

enum CecpCommand {
    XBoard,
    Move(ChessMove),
    Ping(String),
    SetBoard(Board),
    Level(i64, i64, i64),
    Time(f32),
    SetTime(f32),
    Cores(u8),
    Eval,
    Go,
    New,
    MoveNow,
    Force,
    Quit,
    Bench,
    Perf,
    Empty,
}

impl CecpCommand {
    fn new(input: &str) -> Self {
        let input_move = chess::ChessMove::from_str(input);
        if let Ok(m) = input_move {
            return CecpCommand::Move(m);
        }
        let mut split = input.split_ascii_whitespace();
        let token = match split.next() {
            None => {
                return CecpCommand::Empty;
            }
            Some(string) => string,
        };
        match token {
            "xboard" => CecpCommand::XBoard,
            "level" => {
                let mut moves = 0;
                let mut time_left;
                let mut increment = 0;
                if let Some(moves_str) = split.next() {
                    moves = moves_str.parse::<i64>().unwrap_or(0);
                }
                time_left = 0;
                if let Some(time_str) = split.next() {
                    let time_split = time_str.split(':');
                    let mut unit = 60;
                    for time in time_split {
                        time_left += unit * time.parse::<i64>().unwrap_or(0);
                        unit /= 60;
                    }
                }
                if let Some(increment_str) = split.next() {
                    increment = increment_str.parse::<i64>().unwrap_or(0);
                }
                CecpCommand::Level(moves, time_left, increment)
            }
            "ping" => {
                if let Some(number) = split.next() {
                    CecpCommand::Ping(number.to_string())
                } else {
                    CecpCommand::Ping("".to_string())
                }
            }
            "time" => {
                if let Some(seconds) = split.next() {
                    if let Ok(seconds) = seconds.parse::<i64>() {
                        return CecpCommand::Time(seconds as f32 * 0.01);
                    }
                }
                CecpCommand::Empty
            }
            "st" => {
                if let Some(seconds) = split.next() {
                    if let Ok(seconds) = seconds.parse::<f32>() {
                        return CecpCommand::SetTime(seconds);
                    }
                }
                CecpCommand::Empty
            }
            "cores" => {
                if let Some(cores) = split.next() {
                    if let Ok(cores) = cores.parse::<u8>() {
                        return CecpCommand::Cores(cores);
                    }
                }
                CecpCommand::Empty
            }
            "setboard" => {
                let mut fen = "".to_string();
                for token in split {
                    fen.push_str(token);
                    fen.push(' ');
                }
                CecpCommand::SetBoard(chess::Board::from_str(&fen).unwrap())
            }
            "new" => CecpCommand::New,
            "force" => CecpCommand::Force,
            "go" => CecpCommand::Go,
            "?" => CecpCommand::MoveNow,
            "quit" => CecpCommand::Quit,
            "eval" => CecpCommand::Eval,
            "bench" => CecpCommand::Bench,
            "perf" => CecpCommand::Perf,
            _ => CecpCommand::Empty,
        }
    }
}
