use crate::bm::bm_console::BmConsole;
use text_io::read;

mod bm;

#[global_allocator]
#[cfg(feature = "jem")]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {    
    let mut bm_console = BmConsole::new();
    while bm_console.input(read!("{}\n")) {}
}
