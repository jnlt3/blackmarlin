use crate::bm::cecp::cecp_adapter::CecpAdapter;
use text_io::read;

use crate::bm::bm_eval::basic_eval::BasicEval;

use crate::bm::bm_runner::ab_runner::AbRunner;

mod bm;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

type Runner = AbRunner<BasicEval>;

fn main() {
    //bm_nn::testing::train();

    let mut cecp_adapter = CecpAdapter::<BasicEval, Runner>::new();
    while cecp_adapter.input(read!("{}\n")) {}
}
