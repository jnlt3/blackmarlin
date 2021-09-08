/*
use crate::bm::bm_eval::eval::Evaluation;
use std::time::Duration;
use chess::ChessMove;
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};


const MAX_EVAL_DIFF: i32 = 20;

pub trait TimeManager {
    fn new() -> Self;

    fn add(&mut self, eval: Evaluation, best_move: ChessMove, millis: u64);

    fn initiate(&mut self, time_left: u64);

    fn get_time(&self) -> Duration;

    fn clear(&mut self);
}

const MAX_TIME_DIVISOR: u64 = 10;
const MIN_TIME_DIVISOR: u64 = 50;
const TARGET_DEPTH: u32 = 18;

pub struct DepthTimeManager {
    depth: AtomicU32,
    min_time: AtomicU64,
    max_time: AtomicU64,
}

impl TimeManager for DepthTimeManager {
    fn new() -> Self {
        Self {
            min_time: AtomicU64::new(0),
            max_time: AtomicU64::new(0),
            depth: AtomicU32::new(0),
        }
    }

    fn add(&self, _: Evaluation, _: ChessMove, _: u64) {
        self.depth.fetch_add(1, Ordering::SeqCst);
    }

    fn initiate(&mut self, time_left: u64) {
        *self.min_time.as_mut() = time_left / MIN_TIME_DIVISOR;
        *self.max_time.as_mut() = time_left / MAX_TIME_DIVISOR;
    }

    fn get_time(&self) -> Duration {
        let depth_to_go = TARGET_DEPTH.saturating_sub(self.depth.into_inner());
        let min_time = self.min_time.into_inner();
        let max_time = self.max_time.into_inner();
        let time = min_time + ((max_time - min_time) * depth_to_go) / TARGET_DEPTH;
        return Duration::from_millis(time);
    }

    fn clear(&mut self) {
        self.depth = 0;
    }
}

 */
