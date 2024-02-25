use crate::bm::bm_util::eval::Evaluation;
use cozy_chess::{Board, Move};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::ab_runner::MAX_PLY;

const EXPECTED_MOVES: u32 = 64;

const TIME_DEFAULT: Duration = Duration::from_secs(0);
const INC_DEFAULT: Duration = Duration::from_secs(0);

const DEPTH_DEFAULT: u32 = MAX_PLY;

const NODES_DEFAULT: u64 = u64::MAX;

const MOVES_TO_GO_DEFAULT: Option<u32> = None;

#[derive(Debug, Copy, Clone)]
pub enum TimeManagementInfo {
    WTime(Duration),
    BTime(Duration),
    WInc(Duration),
    BInc(Duration),
    MaxDepth(u32),
    MaxNodes(u64),
    MovesToGo(u32),
    MoveTime(Duration),
    Unknown,
}

#[derive(Debug)]
pub struct TimeManager {
    expected_moves: AtomicU32,
    max_duration: AtomicU32,
    base_duration: AtomicU32,
    target_duration: AtomicU32,

    move_stability: AtomicU32,
    prev_eval: AtomicI16,

    prev_move: Mutex<Option<Move>>,
    board: Mutex<Board>,

    infinite: AtomicBool,
    abort_now: AtomicBool,
    no_manage: AtomicBool,

    max_depth: AtomicU32,
    max_nodes: AtomicU64,
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            expected_moves: AtomicU32::new(EXPECTED_MOVES),
            max_duration: AtomicU32::new(0),
            base_duration: AtomicU32::new(0),
            target_duration: AtomicU32::new(0),
            move_stability: AtomicU32::new(0),
            prev_eval: AtomicI16::new(0),
            prev_move: Mutex::new(None),
            board: Mutex::new(Board::default()),
            abort_now: AtomicBool::new(false),
            infinite: AtomicBool::new(true),
            no_manage: AtomicBool::new(true),
            max_depth: AtomicU32::new(DEPTH_DEFAULT),
            max_nodes: AtomicU64::new(NODES_DEFAULT),
        }
    }
}

impl TimeManager {
    pub fn deepen(
        &self,
        thread: usize,
        depth: u32,
        move_nodes: u64,
        nodes: u64,
        eval: Evaluation,
        mv: Move,
    ) {
        let eval = eval.raw();
        let prev_eval = self.prev_eval.load(Ordering::Relaxed);
        self.prev_eval.store(eval, Ordering::Relaxed);
        if thread != 0 || depth <= 4 || self.no_manage.load(Ordering::Relaxed) {
            return;
        }

        let mut prev_move = self.prev_move.lock().unwrap();
        let mut move_stability = self.move_stability.load(Ordering::Relaxed);
        move_stability = match Some(mv) == *prev_move {
            true => (move_stability + 1).min(14),
            false => 0,
        };
        *prev_move = Some(mv);
        self.move_stability.store(move_stability, Ordering::Relaxed);
        let move_stability_factor = (41 - move_stability) as f32 * 0.024;
        let node_factor = (1.0 - move_nodes as f32 / nodes as f32) * 3.42 + 0.52;
        let eval_factor = (prev_eval - eval).clamp(18, 20) as f32 * 0.088;
        let base_duration = self.base_duration.load(Ordering::Relaxed);
        let target_duration =
            base_duration as f32 * move_stability_factor * node_factor * eval_factor;
        self.target_duration
            .store(target_duration as u32, Ordering::Relaxed);
    }

    pub fn initiate(&self, board: &Board, info: &[TimeManagementInfo]) {
        self.abort_now.store(false, Ordering::SeqCst);
        *self.board.lock().unwrap() = board.clone();

        let mut move_cnt = 0;
        board.generate_moves(|piece_moves| {
            move_cnt += piece_moves.into_iter().count();
            false
        });

        let mut infinite = true;

        let mut w_time = TIME_DEFAULT;
        let mut b_time = TIME_DEFAULT;
        let mut w_inc = INC_DEFAULT;
        let mut b_inc = INC_DEFAULT;
        let mut max_depth = DEPTH_DEFAULT;
        let mut max_nodes = NODES_DEFAULT;
        let mut moves_to_go = MOVES_TO_GO_DEFAULT;
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
            cozy_chess::Color::White => (w_time, w_inc),
            cozy_chess::Color::Black => (b_time, b_inc),
        };

        let no_manage = infinite || move_time.is_some();
        self.no_manage.store(no_manage, Ordering::SeqCst);

        if let Some(move_time) = move_time {
            let move_time = move_time.as_millis() as u32;
            self.target_duration.store(move_time, Ordering::SeqCst);
            self.max_duration.store(move_time, Ordering::SeqCst);
        } else if move_cnt == 0 {
            self.target_duration.store(0, Ordering::SeqCst);
        } else {
            let max_time = time.as_millis() as u32 * 4 / 5;
            let expected_moves = moves_to_go.unwrap_or(EXPECTED_MOVES) + 1;
            let default = if move_cnt > 1 {
                (inc.as_millis() as u32 + time.as_millis() as u32 / expected_moves).min(max_time)
            } else {
                0
            };
            self.base_duration.store(default, Ordering::SeqCst);
            self.target_duration.store(default, Ordering::SeqCst);
            self.max_duration.store(max_time, Ordering::SeqCst);
        };
    }

    pub fn abort_now(&self) {
        self.abort_now.store(true, Ordering::SeqCst);
    }

    pub fn abort_search(&self, start: Instant, nodes: u64) -> bool {
        if self.abort_now.load(Ordering::SeqCst) {
            true
        } else {
            (self.max_duration.load(Ordering::SeqCst) < start.elapsed().as_millis() as u32
                && !self.infinite.load(Ordering::SeqCst))
                || self.max_nodes.load(Ordering::SeqCst) <= nodes
        }
    }

    pub fn abort_deepening(&self, start: Instant, depth: u32, nodes: u64) -> bool {
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
        *self.prev_move.lock().unwrap() = None;
        self.abort_now.store(false, Ordering::SeqCst);
        self.no_manage.store(false, Ordering::SeqCst);
        let expected_moves = self.expected_moves.load(Ordering::SeqCst);
        self.expected_moves
            .store(expected_moves.saturating_sub(1), Ordering::SeqCst);
        self.move_stability.store(0, Ordering::Relaxed);
    }
}
