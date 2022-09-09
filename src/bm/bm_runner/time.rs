use crate::bm::bm_util::eval::Evaluation;
use cozy_chess::{Board, Move};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::ab_runner::MAX_PLY;

const EXPECTED_MOVES: u32 = 50;
const MOVE_CHANGE_MARGIN: u32 = 9;

const TIME_DEFAULT: Duration = Duration::from_secs(0);
const INC_DEFAULT: Duration = Duration::from_secs(0);

//We pretty much solve the position if we calculate this deep :D
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
    last_eval: AtomicI16,
    max_duration: AtomicU32,
    normal_duration: AtomicU32,
    target_duration: AtomicU32,

    same_move_depth: AtomicU32,
    move_change_cnt: AtomicU32,
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
            last_eval: AtomicI16::new(0),
            max_duration: AtomicU32::new(0),
            normal_duration: AtomicU32::new(0),
            target_duration: AtomicU32::new(0),
            same_move_depth: AtomicU32::new(0),
            move_change_cnt: AtomicU32::new(0),
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
        thread: u8,
        depth: u32,
        _: u64,
        eval: Evaluation,
        current_move: Move,
        _: Duration,
    ) {
        if thread != 0 || depth <= 4 || self.no_manage.load(Ordering::SeqCst) {
            return;
        }

        let current_eval = eval.raw();
        let last_eval = self.last_eval.load(Ordering::SeqCst);
        let mut time = (self.normal_duration.load(Ordering::SeqCst) * 1000) as f32;

        let mut move_changed = false;
        let prev_move = &mut *self.prev_move.lock().unwrap();
        if let Some(prev_move) = prev_move {
            if *prev_move != current_move {
                move_changed = true;
            }
        }
        *prev_move = Some(current_move);

        let move_change_depth = if move_changed {
            self.move_change_cnt.fetch_add(1, Ordering::SeqCst);
            self.same_move_depth.store(0, Ordering::SeqCst);
            0
        } else {
            self.same_move_depth.fetch_add(1, Ordering::SeqCst)
        };

        let move_change_cnt = self.move_change_cnt.load(Ordering::SeqCst);

        let eval_diff = (current_eval as f32 - last_eval as f32).abs() / 25.0;

        time *= 1.05_f32.powf((eval_diff - 0.1).min(1.0));

        let move_change_factor = 1.05_f32
            .powf(MOVE_CHANGE_MARGIN as f32 - move_change_depth as f32)
            .max(0.4);

        let move_cnt_factor = 1.05_f32.powf(move_change_cnt as f32);

        let max_duration = self.max_duration.load(Ordering::SeqCst) as f32 * 1000.0;
        self.normal_duration
            .store((time.min(max_duration) * 0.001) as u32, Ordering::SeqCst);
        self.target_duration.store(
            (time * 0.001 * move_change_factor * move_cnt_factor).min(max_duration * 0.001) as u32,
            Ordering::SeqCst,
        );
        self.last_eval.store(current_eval, Ordering::SeqCst);
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

        if move_cnt == 0 {
            self.target_duration.store(0, Ordering::SeqCst);
        } else if let Some(move_time) = move_time {
            self.target_duration
                .store(move_time.as_millis() as u32, Ordering::SeqCst);
        } else {
            let max_time = time.as_millis() as u32 * 4 / 5;
            let expected_moves = moves_to_go.unwrap_or(EXPECTED_MOVES) + 1;
            let default = if move_cnt > 1 {
                (inc.as_millis() as u32 + time.as_millis() as u32 / expected_moves).min(max_time)
            } else {
                0
            };
            self.same_move_depth.store(0, Ordering::SeqCst);
            self.move_change_cnt.store(0, Ordering::SeqCst);
            self.normal_duration.store(default, Ordering::SeqCst);
            self.target_duration.store(default, Ordering::SeqCst);
            self.max_duration
                .store(max_time, Ordering::SeqCst);
        };
    }

    pub fn abort_now(&self) {
        self.abort_now.store(true, Ordering::SeqCst);
    }

    pub fn abort_search(&self, start: Instant) -> bool {
        if self.abort_now.load(Ordering::SeqCst) {
            true
        } else {
            self.target_duration.load(Ordering::SeqCst) < start.elapsed().as_millis() as u32
                && !self.infinite.load(Ordering::SeqCst)
        }
    }

    pub fn abort_deepening(&self, start: Instant, depth: u32, nodes: u64) -> bool {
        if self.abort_now.load(Ordering::SeqCst) {
            true
        } else {
            let abort_std = self.target_duration.load(Ordering::SeqCst)
                < (start.elapsed().as_millis() * 10 / 8) as u32
                && !self.infinite.load(Ordering::SeqCst);
            abort_std
                || self.max_depth.load(Ordering::SeqCst) < depth
                || self.max_nodes.load(Ordering::SeqCst) <= nodes
        }
    }

    pub fn clear(&self) {
        *self.prev_move.lock().unwrap() = None;
        self.same_move_depth.store(0, Ordering::SeqCst);
        self.move_change_cnt.store(0, Ordering::SeqCst);
        self.abort_now.store(false, Ordering::SeqCst);
        self.no_manage.store(false, Ordering::SeqCst);
        let expected_moves = self.expected_moves.load(Ordering::SeqCst);
        self.expected_moves
            .store(expected_moves.saturating_sub(1), Ordering::SeqCst);
    }
}
