use crate::bm::bm_console::BmConsole;
use text_io::read;

mod bm;

//TODO: Increase mate finding capabilities and prevent draws by three-fold repetition.
//TODO: Search & Evaluation
//TODO: OpenBench
//TODO: Time Management

/*
This is the main version for the tt_fix branch due to problems caused by previously made mistakes
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
