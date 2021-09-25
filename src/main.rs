use crate::bm::cecp::CecpAdapter;
use text_io::read;

use crate::bm::bm_eval::evaluator::StdEvaluator;

use crate::bm::bm_runner::ab_runner::AbRunner;

mod bm;

#[global_allocator]
#[cfg(feature = "jem")]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

type Runner = AbRunner<StdEvaluator>;

fn main() {
    let mut cecp_adapter = CecpAdapter::<StdEvaluator, Runner>::new();
    while cecp_adapter.input(read!("{}\n")) {}
}
