use crate::bm::bm_eval::eval::Evaluation;
use chess::ChessMove;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
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

const EXPECTED_MOVES: u32 = 80;
const MIN_MOVES: u32 = 25;
const NORMAL_STD_DEV: u32 = 10;
const FACTOR: f64 = 1.0 / NORMAL_STD_DEV as f64;
const POWER: f64 = 1.0;

#[derive(Debug)]
pub struct MainTimeManager {
    expected_moves: AtomicU32,
    evals: Mutex<Vec<(i32, u32)>>,
    normal_duration: AtomicU32,
    max_duration: AtomicU32,
    target_duration: AtomicU32,
}

impl MainTimeManager {
    pub fn new() -> Self {
        Self {
            expected_moves: AtomicU32::new(EXPECTED_MOVES),
            evals: Mutex::new(vec![]),
            normal_duration: AtomicU32::new(0),
            max_duration: AtomicU32::new(0),
            target_duration: AtomicU32::new(0),
        }
    }
}

impl TimeManager for MainTimeManager {
    fn deepen(&self, _: u8, depth: u32, eval: Evaluation, mv: ChessMove, _: Duration) {
        let weight = depth * depth;

        let mut evals = self.evals.lock().unwrap();
        evals.push((eval.raw(), weight));

        let mut sum_weights = 0;
        if depth > 4 {
            evals.iter().rev().for_each(|&(_, weight)| {
                sum_weights += weight;
            });
            let optimal_eval = evals.last().unwrap().0;
            let eval_variance = evals
                .iter()
                .map(|&(eval, weight)| weight as u64 * ((eval - optimal_eval).abs() as u64).pow(2))
                .sum::<u64>()
                / sum_weights as u64;
            let std_dev = (eval_variance as f64).sqrt();

            let time_f64 = self.normal_duration.load(Ordering::SeqCst) as f64;
            let new_time = time_f64 * (std_dev * FACTOR).powf(POWER);
            self.target_duration
                .store(new_time as u32, Ordering::SeqCst);
            self.target_duration
                .fetch_min(self.max_duration.load(Ordering::SeqCst), Ordering::SeqCst);
        }
    }

    fn initiate(&self, time_left: Duration) {
        let percentage_time =
            time_left.as_millis() as u32 / self.expected_moves.load(Ordering::SeqCst);
        self.normal_duration
            .store(percentage_time, Ordering::SeqCst);
        self.target_duration
            .store(percentage_time, Ordering::SeqCst);
        if time_left.as_millis() < 2000 {
            self.max_duration
                .store(time_left.as_millis() as u32 / 10, Ordering::SeqCst)
        } else {
            self.max_duration
                .store(time_left.as_millis() as u32 * 2 / 3, Ordering::SeqCst)
        }
    }

    fn abort(&self, delta_time: Duration) -> bool {
        self.target_duration.load(Ordering::SeqCst) < delta_time.as_millis() as u32
    }

    fn clear(&self) {
        self.evals.lock().unwrap().clear();
        self.expected_moves.fetch_sub(1, Ordering::SeqCst);
        self.expected_moves.fetch_max(MIN_MOVES, Ordering::SeqCst);
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
        self.managers.iter().for_each(|manager| manager.clear());
    }
}
