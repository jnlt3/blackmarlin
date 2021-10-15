use crate::bm::bm_eval::eval::Evaluation;
use chess::ChessMove;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub trait TimeManager: Debug + Send + Sync {
    fn deepen(
        &self,
        thread: u8,
        depth: u32,
        nodes: u32,
        eval: Evaluation,
        best_move: ChessMove,
        delta_time: Duration,
    );

    fn initiate(&self, time_left: Duration, move_cnt: usize);

    fn abort(&self, start: Instant) -> bool;

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
            self.current_depth.load(Ordering::SeqCst) > self.depth.load(Ordering::SeqCst),
            Ordering::SeqCst,
        )
    }
}

impl TimeManager for ConstDepth {
    fn deepen(&self, _: u8, depth: u32, _: u32, _: Evaluation, _: ChessMove, _: Duration) {
        self.current_depth.store(depth, Ordering::SeqCst);
        self.update_abort();
    }

    fn initiate(&self, _: Duration, _: usize) {
        self.abort.store(false, Ordering::SeqCst);
    }

    fn abort(&self, _: Instant) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn clear(&self) {
        self.abort.store(false, Ordering::SeqCst);
        self.current_depth.store(0, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct ConstTime {
    start: Instant,
    target_duration: AtomicU32,
    std_target_duration: AtomicU32,
}

impl ConstTime {
    pub fn new(target_duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            target_duration: AtomicU32::new(target_duration.as_millis() as u32),
            std_target_duration: AtomicU32::new(target_duration.as_millis() as u32),
        }
    }

    pub fn set_duration(&self, duration: Duration) {
        self.target_duration
            .store(duration.as_millis() as u32, Ordering::SeqCst);
    }
}

impl TimeManager for ConstTime {
    fn deepen(&self, _: u8, _: u32, _: u32, _: Evaluation, _: ChessMove, _: Duration) {}

    fn initiate(&self, _: Duration, _: usize) {}

    fn abort(&self, start: Instant) -> bool {
        self.target_duration.load(Ordering::SeqCst) < start.elapsed().as_millis() as u32
    }

    fn clear(&self) {
        self.target_duration.store(
            self.std_target_duration.load(Ordering::SeqCst),
            Ordering::SeqCst,
        );
    }
}

const EXPECTED_MOVES: u32 = 60;
const MIN_MOVES: u32 = 25;

#[derive(Debug)]
pub struct MainTimeManager {
    start: Instant,
    expected_moves: AtomicU32,
    last_eval: AtomicI16,
    max_duration: AtomicU32,
    target_duration: AtomicU32,
}

impl MainTimeManager {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            expected_moves: AtomicU32::new(EXPECTED_MOVES),
            last_eval: AtomicI16::new(0),
            max_duration: AtomicU32::new(0),
            target_duration: AtomicU32::new(0),
        }
    }
}

impl TimeManager for MainTimeManager {
    fn deepen(&self, _: u8, depth: u32, _: u32, eval: Evaluation, _: ChessMove, _: Duration) {
        if depth <= 4 {
            return;
        }
        let current_eval = eval.raw();
        let last_eval = self.last_eval.load(Ordering::SeqCst);
        let mut time = (self.target_duration.load(Ordering::SeqCst) * 1000) as f32;
        time *= 1.1_f32.powf((current_eval - last_eval).abs() as f32 / 50.0);
        let time = time.min(self.max_duration.load(Ordering::SeqCst) as f32 * 1000.0);
        self.target_duration
            .store((time * 0.001) as u32, Ordering::SeqCst);
        self.last_eval.store(current_eval, Ordering::SeqCst);
    }

    fn initiate(&self, time_left: Duration, move_cnt: usize) {
        if move_cnt == 0 {
            self.target_duration.store(0, Ordering::SeqCst);
        } else {
            self.target_duration.store(
                time_left.as_millis() as u32 / self.expected_moves.load(Ordering::SeqCst),
                Ordering::SeqCst,
            );
            self.max_duration
                .store(time_left.as_millis() as u32 * 2 / 3, Ordering::SeqCst);
        };
    }

    fn abort(&self, start: Instant) -> bool {
        self.target_duration.load(Ordering::SeqCst) < start.elapsed().as_millis() as u32
    }

    fn clear(&self) {
        self.expected_moves.fetch_sub(1, Ordering::SeqCst);
        self.expected_moves.fetch_max(MIN_MOVES, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct ManualAbort {
    abort: AtomicBool,
}

impl ManualAbort {
    pub fn new() -> Self {
        Self {
            abort: AtomicBool::new(false),
        }
    }

    pub fn quick_abort(&self) {
        self.abort.store(true, Ordering::SeqCst);
    }
}

impl TimeManager for ManualAbort {
    fn deepen(&self, _: u8, _: u32, _: u32, _: Evaluation, _: ChessMove, _: Duration) {}

    fn initiate(&self, _: Duration, _: usize) {
        self.abort.store(false, Ordering::SeqCst);
    }

    fn abort(&self, _: Instant) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn clear(&self) {}
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
        nodes: u32,
        eval: Evaluation,
        best_move: ChessMove,
        delta_time: Duration,
    ) {
        self.managers[self.mode.load(Ordering::SeqCst)]
            .deepen(thread, depth, nodes, eval, best_move, delta_time);
    }

    fn initiate(&self, time_left: Duration, move_cnt: usize) {
        self.managers[self.mode.load(Ordering::SeqCst)].initiate(time_left, move_cnt);
    }

    fn abort(&self, start: Instant) -> bool {
        self.managers[self.mode.load(Ordering::SeqCst)].abort(start)
    }

    fn clear(&self) {
        self.managers.iter().for_each(|manager| manager.clear());
    }
}

#[derive(Debug)]
pub struct Diagnostics<Inner: TimeManager> {
    manager: Arc<Inner>,
    data: Mutex<Vec<(u32, u32)>>,
}

impl<Inner: TimeManager> Diagnostics<Inner> {
    pub fn new(manager: Arc<Inner>) -> Diagnostics<Inner> {
        Self {
            manager,
            data: Mutex::new(vec![]),
        }
    }

    pub fn get_data(&self) -> &Mutex<Vec<(u32, u32)>> {
        &self.data
    }
}

impl<Inner: TimeManager> TimeManager for Diagnostics<Inner> {
    fn deepen(
        &self,
        thread: u8,
        depth: u32,
        nodes: u32,
        eval: Evaluation,
        best_move: ChessMove,
        delta_time: Duration,
    ) {
        self.manager
            .deepen(thread, depth, nodes, eval, best_move, delta_time);
        self.data.lock().unwrap().push((nodes, depth));
    }

    fn initiate(&self, time_left: Duration, move_cnt: usize) {
        self.manager.initiate(time_left, move_cnt);
    }

    fn abort(&self, start: Instant) -> bool {
        self.manager.abort(start)
    }

    fn clear(&self) {
        self.manager.clear();
    }
}