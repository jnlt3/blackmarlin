use crate::bm::bm_console::BmConsole;
use text_io::read;

mod bm;

//TODO: Increase mate finding capabilities and prevent draws by three-fold repetition.
//TODO: Search & Evaluation
//TODO: OpenBench
//TODO: Time Management


/*
This is a work around version to compare two versions on even time control
The tt_fix branch was made from fail_tm causing multiple differences between the main branch
*/

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
