use super::uci::UciAdapter;

#[cfg(feature = "data")]
mod gen_eval;
#[cfg(feature = "trace")]
mod gen_fen;
#[cfg(feature = "trace")]
mod grad;

pub struct BmConsole {
    uci: UciAdapter,
}

impl BmConsole {
    pub fn new() -> Self {
        Self {
            uci: UciAdapter::new(),
        }
    }

    pub fn input(&mut self, command: String) -> bool {
        if command.is_empty() {
            return false;
        }
        #[cfg(any(feature = "trace", feature = "data"))]
        if command.starts_with("!") {
            let (command, options) = Self::parse(&command[1..]);
            let command: &str = &command;
            match command {
                #[cfg(feature = "trace")]
                "gen" => Self::gen(options),
                #[cfg(feature = "trace")]
                "tune" => Self::tune(options),
                #[cfg(feature = "data")]
                "data" => Self::data(),
                _ => {}
            }
            return true;
        }
        self.uci.input(command)
    }

    #[cfg(feature = "data")]
    fn data() {
        gen_eval::gen_eval();
    }

    #[cfg(feature = "trace")]
    fn tune(options: Vec<(String, String)>) {
        use std::{collections::HashMap, str::FromStr};

        use chess::Board;

        use crate::bm::{
            bm_console::gen_fen::DataPoint,
            bm_eval::evaluator::{EvalTrace, StdEvaluator},
        };

        let option_param = options.iter().find(|(key, _)| key == "input");
        if option_param.is_none() {
            println!("error in parsing input file");
            return;
        }
        let input_file = &option_param.unwrap().1;

        let content = std::fs::read_to_string(input_file).unwrap();
        let mut traces = HashMap::<EvalTrace, (f64, usize, f64)>::new();
        let mut eval = StdEvaluator::new();
        for line in content.lines() {
            let board;
            let result;
            let weight;
            if line.contains(",") {
                let mut split = line.split(",");
                board = Board::from_str(split.next().unwrap()).unwrap();
                result = split.next().unwrap().trim().parse::<f64>().unwrap();
                weight = split.next().unwrap().trim().parse::<f64>().unwrap();
            } else {
                let mut split = line.split(" [");
                board = Board::from_str(split.next().unwrap()).unwrap();
                result = split
                    .next()
                    .unwrap()
                    .replace("]", "")
                    .parse::<f64>()
                    .unwrap();
                weight = 1.0;
            }
            eval.evaluate(&board);
            let trace = eval.get_trace().clone();

            if let Some(value) = traces.get_mut(&trace) {
                value.0 += result;
                value.1 += 1;
                value.2 += weight;
            } else {
                traces.insert(trace, (result, 1, weight));
            }
        }
        let traces = traces
            .into_iter()
            .map(|(trace, result)| DataPoint {
                trace,
                result: result.0 / result.1 as f64,
                weight: result.2 / result.1 as f64,
            })
            .collect::<Vec<_>>();
        grad::tune(&traces);
    }

    #[cfg(feature = "trace")]
    fn gen(options: Vec<(String, String)>) {
        let option_param = options.iter().find(|(key, _)| key == "input");
        if option_param.is_none() {
            println!("error in parsing input file");
            return;
        }
        let input_files = option_param.unwrap().1.split(" ").collect::<Vec<_>>();

        let option_param = options.iter().find(|(key, _)| key == "output");
        if option_param.is_none() {
            println!("error in parsing output file");
            return;
        }
        let output_file = &option_param.unwrap().1;

        let mut all_fens = "".to_string();
        for input_file in input_files.chunks(2) {
            let weight = input_file[1].parse::<f64>().unwrap();
            let input_file = input_file[0];
            let pgns = std::fs::read_to_string(input_file.trim()).unwrap();
            let fens = gen_fen::fens(&pgns, weight);
            all_fens += &fens;
        }
        std::fs::write(output_file, all_fens).unwrap();
    }

    fn parse(command: &str) -> (String, Vec<(String, String)>) {
        let split = command.split(' ').collect::<Vec<_>>();

        let main_command = split[0].to_string();

        let mut option = "".to_string();
        let mut param = "".to_string();

        let mut options = vec![];

        for token in split.into_iter() {
            if let Some(token) = token.strip_prefix('-') {
                if !option.is_empty() && !param.is_empty() {
                    options.push((option, param.trim().to_string()));
                }
                option = token.to_string();
                param = "".to_string();
            } else {
                param += &(token.to_string() + " ");
            }
        }
        if !option.is_empty() && !param.is_empty() {
            options.push((option, param.trim().to_string()));
        }
        (main_command, options)
    }
}
