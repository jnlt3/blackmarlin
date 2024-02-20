mod uci;
pub mod bench;
pub mod command;

use uci::UciAdapter;

fn main() {
    let mut uci = UciAdapter::new();
    let mut args = String::new();
    for arg in std::env::args().skip(1) {
        args.push_str(&arg);
        args.push_str(" ");
    }
    if !args.is_empty() {
        uci.input(&args);
        return;
    }
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        if buffer.is_empty() {
            return;
        }
        let (command, _new_line) = buffer.split_at(buffer.len() - 1);
        if !uci.input(command) {
            return;
        }
    }
}
