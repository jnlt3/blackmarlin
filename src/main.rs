use crate::bm::bm_console::BmConsole;
use text_io::read;

mod bm;

#[global_allocator]
#[cfg(feature = "jem")]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

//TODO: Increase mate finding capabilities and prevent draws by three-fold repetition.
//TODO: Search & Evaluation
//TODO: OpenBench
//TODO: Time Management

fn main() {
    let mut bm_console = BmConsole::new();
    for arg in std::env::args() {
        if arg.trim() == "bench" {
            bm_console.input("bench".to_string());
            return;
        }
    }
    while bm_console.input(read!("{}\n")) {}
}
