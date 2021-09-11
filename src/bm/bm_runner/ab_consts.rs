pub const KILLER_MOVE_CNT: usize = 2;
pub const THREAT_MOVE_CNT: usize = 1;

pub const IID_DEPTH: u32 = 5;

pub const F_PRUNE_DEPTH: u32 = 6;
pub const F_PRUNE_THRESHOLD_BASE: i32 = 200;
pub const F_PRUNE_THRESHOLD_FACTOR: i32 = 200;
pub const DO_F_PRUNE: bool = true;

pub const NULL_MOVE_REDUCTION_BASE: u32 = 1;
pub const NULL_MOVE_REDUCTION_FACTOR: u32 = 1;
pub const NULL_MOVE_REDUCTION_DIVISOR: u32 = 4;
pub const DO_NULL_MOVE_REDUCTION: bool = true;

pub const IID_BASE: u32 = 1;
pub const IID_FACTOR: u32 = 1;
pub const IID_DIVISOR: u32 = 4;
pub const DO_IID: bool = true;

pub const LMR_BASE: f32 = 0.75;
pub const LMR_DIV: f32 = 1.25;
pub const LMR_PV: u32 = 1;
pub const LMR_DEPTH: u32 = 0;
pub const DO_LMR: bool = true;

pub const LMP_DEPTH: u32 = 10;
pub const LMP_OFFSET: f32 = 2.0;
pub const LMP_FACTOR: f32 = 0.88;
pub const DO_LMP: bool = false;

pub const QUIESCENCE_SEARCH_DEPTH: u32 = 30;
pub const DELTA_MARGIN: i32 = 1000;
pub const DO_DELTA_PRUNE: bool = true;
pub const DO_SEE_PRUNE: bool = true;

pub const FAIL_CNT: u8 = 5;
pub const WINDOW_START: i32 = 25;
pub const WINDOW_FACTOR: i32 = 2;
pub const WINDOW_DIVISOR: i32 = 1;
pub const WINDOW_ADD: i32 = 5;