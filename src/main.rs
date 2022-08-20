use crate::bm::bm_console::BmConsole;
use text_io::read;

mod bm;

fn main() {
    let mut bm_console = BmConsole::new();
    let mut args = String::new();
    for arg in std::env::args().skip(1) {
        args.push_str(&arg);
        args.push_str(" ");
    }
    if !args.is_empty() {
        bm_console.input(args);
        return;
    }
    while bm_console.input(read!("{}\n")) {}
}
