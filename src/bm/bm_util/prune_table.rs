use crate::bm::bm_eval::eval::Evaluation;

const EVAL_DIV: i16 = 50;
const EVAL_SEGMENTS: usize = 15;

const HISTORY_DIV: i16 = 64;
const HISTORY_SEGMENTS: usize = 17;

const IN_CHECK_SEGMENTS: usize = 2;

const DEPTH_SEGMENTS: usize = 8;

#[derive(Debug, Clone)]
pub struct QuietPruneTable {
    table: [[[[Stats; EVAL_SEGMENTS]; HISTORY_SEGMENTS]; IN_CHECK_SEGMENTS]; DEPTH_SEGMENTS],
}

impl QuietPruneTable {
    pub fn new() -> Self {
        let stats = Stats {
            success: 100,
            visits: 1,
        };
        Self {
            table: [[[[stats; EVAL_SEGMENTS]; HISTORY_SEGMENTS]; IN_CHECK_SEGMENTS];
                DEPTH_SEGMENTS],
        }
    }

    pub fn visit(
        &mut self,
        eval: Evaluation,
        alpha: Evaluation,
        history: i16,
        in_check: bool,
        depth: u32,
        success: bool,
    ) {
        let depth = depth as usize;
        assert!(depth <= DEPTH_SEGMENTS);
        let eval_index = ((eval - alpha).raw() / EVAL_DIV + (EVAL_SEGMENTS / 2) as i16)
            .max(0)
            .min(EVAL_SEGMENTS as i16 - 1) as usize;
        let history_index = (history / HISTORY_DIV + (HISTORY_SEGMENTS / 2) as i16) as usize;
        let in_check_index = in_check as usize;
        let stats = &mut self.table[depth - 1][in_check_index][history_index][eval_index];
        stats.visits += 1;
        stats.success += success as u32;
    }

    pub fn predict_success(
        &self,
        eval: Evaluation,
        alpha: Evaluation,
        history: i16,
        in_check: bool,
        depth: u32,
        success_rate: u32,
    ) -> bool {
        let depth = depth as usize;
        let eval_index = ((eval - alpha).raw() / EVAL_DIV + (EVAL_SEGMENTS / 2) as i16)
            .max(0)
            .min(EVAL_SEGMENTS as i16 - 1) as usize;
        let history_index = (history / HISTORY_DIV + (HISTORY_SEGMENTS / 2) as i16)
            .max(0)
            .min(HISTORY_SEGMENTS as i16 - 1) as usize;
        let in_check_index = in_check as usize;
        let stats = &self.table[depth - 1][in_check_index][history_index][eval_index];
        /*
        (x / y) * 100 > z
        */
        stats.success * 100 / stats.visits > success_rate
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    success: u32,
    visits: u32,
}
