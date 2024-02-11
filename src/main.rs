use crate::bm::bm_console::BmConsole;

pub mod bm;

fn main() {
    let mut bm_console = BmConsole::new();
    let mut args = String::new();
    for arg in std::env::args().skip(1) {
        args.push_str(&arg);
        args.push_str(" ");
    }
    if !args.is_empty() {
        bm_console.input(&args);
        return;
    }
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        if buffer.is_empty() {
            return;
        }
        let (command, _new_line) = buffer.split_at(buffer.len() - 1);
        if !bm_console.input(command) {
            return;
        }
    }
}
