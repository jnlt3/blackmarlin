use crate::bm::util::eval::Evaluation;
use cozy_chess::{Board, Move};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchStats {
    delta_time: u128,
    depth: u32,
    evaluation: Option<Evaluation>,
    best_move: Option<Move>,
}

impl SearchStats {
    pub fn new(
        delta_time: u128,
        depth: u32,
        evaluation: Option<Evaluation>,
        best_move: Option<Move>,
    ) -> Self {
        Self {
            delta_time,
            depth,
            evaluation,
            best_move,
        }
    }
}

pub trait SearchMode {
    fn new(board: &Board) -> Self;

    fn push(&mut self, data: SearchStats);

    fn complete(&mut self);
}

pub struct Run;

impl SearchMode for Run {
    fn new(_: &Board) -> Self {
        Self {}
    }

    fn push(&mut self, _: SearchStats) {}

    fn complete(&mut self) {}
}

pub struct Debug {
    fen: String,
    info: Vec<SearchStats>,
}

impl SearchMode for Debug {
    fn new(board: &Board) -> Self {
        Self {
            fen: board.to_string(),
            info: vec![],
        }
    }

    fn push(&mut self, data: SearchStats) {
        self.info.push(data);
    }

    fn complete(&mut self) {
        let mut position = format("Position: ");
        position.push_str(&self.fen);

        let mut time_buffer = format("Time: ");
        let mut depth_buffer = format("Depth: ");
        let mut eval_buffer = format("Eval: ");
        let mut move_buffer = format("Move: ");

        for stats in &self.info {
            time_buffer.push_str(&format(stats.delta_time));
            depth_buffer.push_str(&format(stats.depth));
            if let Some(eval) = stats.evaluation {
                eval_buffer.push_str(&format(eval.raw()));
            } else {
                eval_buffer.push_str(&format("None"));
            }
            if let Some(best_move) = stats.best_move {
                move_buffer.push_str(&format(best_move));
            } else {
                move_buffer.push_str(&format("None"));
            }
        }
        position.push('\n');
        time_buffer.push('\n');
        depth_buffer.push('\n');
        eval_buffer.push('\n');
        move_buffer.push_str(&"\n".repeat(3));

        position.push_str(&time_buffer);
        position.push_str(&depth_buffer);
        position.push_str(&eval_buffer);
        position.push_str(&move_buffer);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("./data/debug.txt")
        {
            if let Err(e) = file.write_all(position.as_bytes()) {
                println!("# {}", e);
            }
        }
        self.info.clear();
    }
}

fn format<T: Display>(value: T) -> String {
    let mut string = format!("{} ", value);
    extend(&mut string, 15);
    string
}

fn extend(string: &mut String, len: usize) {
    if string.len() < len {
        string.push_str(&" ".repeat(len - string.len()));
    }
}

pub trait GuiInfo {
    fn new() -> Self;

    fn print_info(
        &self,
        sel_depth: u32,
        depth: u32,
        eval: Evaluation,
        elapsed: Duration,
        node_cnt: u64,
        pv: &[Move],
    );
}

#[derive(Debug, Clone)]
pub struct NoInfo;

impl GuiInfo for NoInfo {
    fn new() -> Self {
        Self {}
    }

    fn print_info(&self, _: u32, _: u32, _: Evaluation, _: Duration, _: u64, _: &[Move]) {}
}

#[derive(Debug, Clone)]
pub struct UciInfo;

impl GuiInfo for UciInfo {
    fn new() -> Self {
        Self {}
    }

    fn print_info(
        &self,
        seldepth: u32,
        depth: u32,
        eval: Evaluation,
        elapsed: Duration,
        node_cnt: u64,
        pv: &[Move],
    ) {
        let eval_str = if eval.is_mate() {
            format!("mate {}", eval.mate_in().unwrap())
        } else {
            format!("cp {}", eval.raw())
        };
        let nps = (node_cnt as u128 * 1000) / elapsed.as_millis().max(1);
        let mut buffer = String::new();
        buffer += &format!(
            "info depth {} seldepth {} score {} time {} nodes {} nps {} pv",
            depth,
            seldepth,
            eval_str,
            elapsed.as_millis(),
            node_cnt,
            nps
        );
        for make_move in pv {
            buffer += &format!(" {}", make_move);
        }
        println!("{}", buffer);
    }
}
