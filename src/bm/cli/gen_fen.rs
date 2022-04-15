#[cfg(feature = "trace")]
use crate::bm::bm_eval::evaluator::EvalTrace;

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub trace: EvalTrace,
    pub result: f64,
    pub weight: f64,
}
