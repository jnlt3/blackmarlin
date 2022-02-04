pub const KILLER_MOVE_CNT: usize = 2;
pub const THREAT_MOVE_CNT: usize = 1;

pub const F_PRUNE_THRESHOLD: i16 = 100;
pub const DO_F_PRUNE: bool = true;

pub const REV_F_PRUNE_DEPTH: u32 = 7;
pub const REV_F_PRUNE_THRESHOLD_BASE: i16 = 0;
pub const REV_F_PRUNE_THRESHOLD_FACTOR: i16 = 50;
pub const DO_REV_F_PRUNE: bool = true;

pub const NULL_MOVE_REDUCTION_BASE: u32 = 2;
pub const NULL_MOVE_REDUCTION_FACTOR: u32 = 1;
pub const NULL_MOVE_REDUCTION_DIVISOR: u32 = 4;
pub const NULL_MOVE_PRUNE_DEPTH: u32 = 5;
pub const DO_NULL_MOVE_REDUCTION: bool = true;

pub const IID_DEPTH: u32 = 5;
pub const IID_BASE: u32 = 1;
pub const IID_FACTOR: u32 = 1;
pub const IID_DIVISOR: u32 = 4;
pub const DO_IID: bool = false;

pub const LMR_BASE: f32 = 0.75;
pub const LMR_DIV: f32 = 1.25;
pub const LMR_DEPTH: u32 = 1;
pub const DO_LMR: bool = true;

pub const LMP_DEPTH: u32 = 64;
pub const LMP_OFFSET: f32 = 3.0;
pub const LMP_FACTOR: f32 = 0.5;
pub const IMPROVING_DIVISOR: f32 = 1.5;
pub const DO_LMP: bool = true;

pub const QUIESCENCE_SEARCH_DEPTH: u32 = 30;
pub const DELTA_MARGIN: i16 = 1000;
pub const DO_DELTA_PRUNE: bool = true;

pub const FAIL_CNT: u8 = 10;
pub const WINDOW_START: i16 = 25;
pub const WINDOW_FACTOR: i16 = 1;
pub const WINDOW_DIVISOR: i16 = 4;
pub const WINDOW_ADD: i16 = 5;

pub const HISTORY_REDUCTION_DIVISOR: i16 = 192;
