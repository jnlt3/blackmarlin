use crate::bm::bm_eval::eval::Evaluation;
use chess::ChessMove;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub trait TimeManager: Debug + Send + Sync {
    fn deepen(
        &self,
        thread: u8,
        depth: u32,
        eval: Evaluation,
        best_move: ChessMove,
        delta_time: Duration,
    );

    fn initiate(&self, time_left: Duration);

    fn abort(&self, delta_time: Duration) -> bool;

    fn clear(&self);
}

#[derive(Debug, Copy, Clone)]
pub struct Percentage {
    numerator: u32,
    denominator: u32,
}

impl Percentage {
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn get(&self, duration: u32) -> u32 {
        duration * self.numerator / self.denominator
    }
}

#[derive(Debug)]
pub struct PercentTime {
    target_duration: AtomicU32,
    max_duration: AtomicU32,
    percentage: Percentage,
}

impl PercentTime {
    pub fn new(percentage: Percentage, max_duration: Duration) -> Self {
        Self {
            target_duration: AtomicU32::new(0),
            percentage,
            max_duration: AtomicU32::new(max_duration.as_millis() as u32),
        }
    }
}

impl TimeManager for PercentTime {
    fn deepen(&self, _: u8, _: u32, _: Evaluation, _: ChessMove, _: Duration) {}

    fn initiate(&self, time_left: Duration) {
        self.target_duration.store(
            self.percentage
                .get(time_left.as_millis() as u32)
                .min(self.max_duration.load(Ordering::SeqCst)),
            Ordering::SeqCst,
        );
    }

    fn abort(&self, delta_time: Duration) -> bool {
        delta_time.as_millis() as u32 > self.target_duration.load(Ordering::SeqCst)
    }

    fn clear(&self) {}
}

#[derive(Debug)]
pub struct ConstDepth {
    current_depth: AtomicU32,
    depth: AtomicU32,
    abort: AtomicBool,
}

impl ConstDepth {
    pub fn new(depth: u32) -> Self {
        Self {
            current_depth: AtomicU32::new(0),
            depth: AtomicU32::new(depth),
            abort: AtomicBool::new(false),
        }
    }

    pub fn set_depth(&self, depth: u32) {
        self.depth.store(depth, Ordering::SeqCst);
        self.update_abort();
    }

    fn update_abort(&self) {
        self.abort.store(
            self.current_depth.load(Ordering::SeqCst) >= self.depth.load(Ordering::SeqCst),
            Ordering::SeqCst,
        )
    }
}

impl TimeManager for ConstDepth {
    fn deepen(&self, _: u8, depth: u32, _: Evaluation, _: ChessMove, _: Duration) {
        self.current_depth.store(depth, Ordering::SeqCst);
        self.update_abort();
    }

    fn initiate(&self, _: Duration) {}

    fn abort(&self, _: Duration) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn clear(&self) {
        self.abort.store(false, Ordering::SeqCst);
        self.current_depth.store(0, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct ConstTime {
    target_duration: AtomicU32,
}

impl ConstTime {
    pub fn new(target_duration: Duration) -> Self {
        Self {
            target_duration: AtomicU32::new(target_duration.as_millis() as u32),
        }
    }

    pub fn set_duration(&self, duration: Duration) {
        self.target_duration
            .store(duration.as_millis() as u32, Ordering::SeqCst);
    }
}

impl TimeManager for ConstTime {
    fn deepen(&self, _: u8, _: u32, _: Evaluation, _: ChessMove, _: Duration) {}

    fn initiate(&self, _: Duration) {}

    fn abort(&self, delta_time: Duration) -> bool {
        self.target_duration.load(Ordering::SeqCst) < delta_time.as_millis() as u32
    }

    fn clear(&self) {
        self.target_duration.store(u32::MAX, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct CompoundTimeManager {
    managers: Box<[Arc<dyn TimeManager>]>,
    mode: AtomicUsize,
}

impl CompoundTimeManager {
    pub fn new(managers: Box<[Arc<dyn TimeManager>]>, initial_mode: usize) -> Self {
        Self {
            managers,
            mode: AtomicUsize::new(initial_mode),
        }
    }

    pub fn set_mode(&self, mode: usize) {
        self.mode.store(mode, Ordering::SeqCst);
    }
}

impl TimeManager for CompoundTimeManager {
    fn deepen(
        &self,
        thread: u8,
        depth: u32,
        eval: Evaluation,
        best_move: ChessMove,
        delta_time: Duration,
    ) {
        self.managers[self.mode.load(Ordering::SeqCst)]
            .deepen(thread, depth, eval, best_move, delta_time);
    }

    fn initiate(&self, time_left: Duration) {
        self.managers[self.mode.load(Ordering::SeqCst)].initiate(time_left);
    }

    fn abort(&self, delta_time: Duration) -> bool {
        self.managers[self.mode.load(Ordering::SeqCst)].abort(delta_time)
    }

    fn clear(&self) {
        self.managers[self.mode.load(Ordering::SeqCst)].clear();
    }
}
