use std::str::FromStr;

use crate::bm::bm_console::BmConsole;
use cozy_chess::Board;
use text_io::read;

mod bm;

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
