use crate::bm::bm_eval::eval::Evaluation;
use chess::{Board, ChessMove, MoveGen};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const EXPECTED_MOVES: u32 = 40;

const TIME_DEFAULT: Duration = Duration::from_secs(0);
const INC_DEFAULT: Duration = Duration::from_secs(0);

//We pretty much solve the position if we calculate this deep :D
const DEPTH_DEFAULT: u32 = 255;

//TODO: consider u64 as more than an hour of analysis will most likely cause an overflow
const NODES_DEFAULT: u32 = u32::MAX;

const MOVES_TO_GO_DEFAULT: Option<u32> = None;

const TIME: Duration = Duration::from_secs(0);

#[derive(Debug, Copy, Clone)]
pub enum TimeManagementInfo {
    WTime(Duration),
    BTime(Duration),
    WInc(Duration),
    BInc(Duration),
    MaxDepth(u32),
    MaxNodes(u32),
    MovesToGo(u32),
    MoveTime(Duration),
    Time(Duration),
    Unknown,
}

#[derive(Debug)]
pub struct TimeManager {
    start: Instant,
    expected_moves: AtomicU32,
    last_eval: AtomicI16,
    max_duration: AtomicU32,
    normal_duration: AtomicU32,
    target_duration: AtomicU32,
    board: Mutex<Board>,

    infinite: AtomicBool,
    abort_now: AtomicBool,
    no_manage: AtomicBool,

    max_depth: AtomicU32,
    max_nodes: AtomicU32,
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            expected_moves: AtomicU32::new(EXPECTED_MOVES),
            last_eval: AtomicI16::new(0),
            max_duration: AtomicU32::new(0),
            normal_duration: AtomicU32::new(0),
            target_duration: AtomicU32::new(0),
            board: Mutex::new(Board::default()),
            abort_now: AtomicBool::new(false),
            infinite: AtomicBool::new(true),
            no_manage: AtomicBool::new(true),
            max_depth: AtomicU32::new(DEPTH_DEFAULT),
            max_nodes: AtomicU32::new(NODES_DEFAULT),
        }
    }
}

impl TimeManager {
    pub fn deepen(&self, _: u8, depth: u32, _: u32, eval: Evaluation, _: ChessMove, _: Duration) {
        if depth <= 4 || self.no_manage.load(Ordering::SeqCst) {
            return;
        }

        let current_eval = eval.raw();
        let last_eval = self.last_eval.load(Ordering::SeqCst);
        let mut time = (self.normal_duration.load(Ordering::SeqCst) * 1000) as f32;

        let mut eval_diff = (current_eval as f32 - last_eval as f32) / 25.0;

        if eval_diff < 0.0 {
            eval_diff *= 1.5;
        };
        eval_diff = eval_diff.max(-1.0).min(1.0).abs();

        time *= 1.05_f32.powf(eval_diff);

        let time = time.min(self.max_duration.load(Ordering::SeqCst) as f32 * 1000.0);
        self.normal_duration
            .store((time * 0.001) as u32, Ordering::SeqCst);
        self.target_duration
            .store((time * 0.001) as u32, Ordering::SeqCst);
        self.last_eval.store(current_eval, Ordering::SeqCst);
    }

    pub fn initiate(&self, board: &Board, info: &[TimeManagementInfo]) {
        self.abort_now.store(false, Ordering::SeqCst);
        *self.board.lock().unwrap() = *board;
        let move_cnt = MoveGen::new_legal(board).into_iter().count();

        let mut infinite = true;

        let mut w_time = TIME_DEFAULT;
        let mut b_time = TIME_DEFAULT;
        let mut w_inc = INC_DEFAULT;
        let mut b_inc = INC_DEFAULT;
        let mut max_depth = DEPTH_DEFAULT;
        let mut max_nodes = NODES_DEFAULT;
        let mut moves_to_go = MOVES_TO_GO_DEFAULT;
        let mut overall_time = TIME;
        let mut move_time = None;

        for info in info {
            match info {
                TimeManagementInfo::WTime(time) => {
                    w_time = *time;
                    infinite = false;
                }
                TimeManagementInfo::BTime(time) => {
                    b_time = *time;
                    infinite = false;
                }
                TimeManagementInfo::WInc(time) => {
                    w_inc = *time;
                }
                TimeManagementInfo::BInc(time) => {
                    b_inc = *time;
                }
                TimeManagementInfo::MaxDepth(depth) => {
                    max_depth = *depth;
                }
                TimeManagementInfo::MaxNodes(nodes) => {
                    max_nodes = *nodes;
                }
                TimeManagementInfo::MovesToGo(moves) => {
                    moves_to_go = Some(*moves);
                }
                TimeManagementInfo::Time(time) => {
                    overall_time = *time;
                }
                TimeManagementInfo::MoveTime(time) => {
                    move_time = Some(*time);
                    infinite = false;
                }
                _ => {}
            }
        }
        self.infinite.store(infinite, Ordering::SeqCst);
        self.max_depth.store(max_depth, Ordering::SeqCst);
        self.max_nodes.store(max_nodes, Ordering::SeqCst);

        let (time, inc) = match board.side_to_move() {
            chess::Color::White => (w_time, w_inc),
            chess::Color::Black => (b_time, b_inc),
        };

        let no_manage = infinite || move_time.is_some();
        self.no_manage.store(no_manage, Ordering::SeqCst);

        if move_cnt == 0 {
            self.target_duration.store(0, Ordering::SeqCst);
        } else {
            let expected_moves = moves_to_go.unwrap_or(EXPECTED_MOVES) + 1;
            let default = if MoveGen::new_legal(board).len() > 1 {
                inc.as_millis() as u32 + time.as_millis() as u32 / expected_moves
            } else {
                0
            };
            self.normal_duration.store(default, Ordering::SeqCst);
            self.target_duration.store(default, Ordering::SeqCst);
            self.max_duration
                .store(time.as_millis() as u32 / 3, Ordering::SeqCst);
        };
    }

    pub fn abort_now(&self) {
        self.abort_now.store(true, Ordering::SeqCst);
    }

    pub fn abort(&self, start: Instant, depth: u32, nodes: u32) -> bool {
        if self.abort_now.load(Ordering::SeqCst) {
            true
        } else {
            let abort_std = self.target_duration.load(Ordering::SeqCst)
                < start.elapsed().as_millis() as u32
                && !self.infinite.load(Ordering::SeqCst);
            abort_std
                || self.max_depth.load(Ordering::SeqCst) < depth
                || self.max_nodes.load(Ordering::SeqCst) <= nodes
        }
    }

    pub fn clear(&self) {
        self.abort_now.store(false, Ordering::SeqCst);
        self.no_manage.store(false, Ordering::SeqCst);
        let expected_moves = self.expected_moves.load(Ordering::SeqCst);
        self.expected_moves
            .store(expected_moves.saturating_sub(1), Ordering::SeqCst);
    }
}
