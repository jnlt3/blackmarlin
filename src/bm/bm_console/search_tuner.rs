use std::{str::FromStr, sync::Arc};

use cozy_chess::{Board, Move};

use crate::bm::bm_runner::{
    ab_runner::AbRunner,
    config::{NoInfo, Run},
    time::{TimeManagementInfo, TimeManager},
};

pub struct SearchTuner {
    parameters: Vec<Box<dyn Fn(i16) -> ()>>,
    ranges: Vec<(i16, i16)>,
}

macro_rules! add_closure {
    ($tuner: expr, $name: ident: $var_type: ty = range($min: expr, $max: expr);) => {{
        let tuner: &mut SearchTuner = $tuner;
        use crate::bm::bm_search::search::$name;
        let closure = Box::new(|value: i16| {
            unsafe { $name = value as $var_type };
        });
        tuner.parameters.push(closure);
        tuner.ranges.push(($min, $max));
    }};
}

macro_rules! fill_vec {
    ($tuner: expr, $name: ident: $var_type: ty = range($min: expr, $max: expr);) => {
        add_closure!($tuner, $name: $var_type = range($min, $max););
    };
    ($tuner: expr, $name: ident: $var_type: ty = range($min: expr, $max: expr);
    $($name_1: ident: $var_type_1: ty = range($min_1: expr, $max_1: expr);)+) => {
        add_closure!($tuner, $name: $var_type = range($min, $max););
        fill_vec!($tuner, $($name_1: $var_type_1 = range($min_1, $max_1);)*);
    };
}

macro_rules! search_tuner {
    {$name: ident: $var_type: ty = range($min: expr, $max: expr);
    $($name_1: ident: $var_type_1: ty = range($min_1: expr, $max_1: expr);)*} => {
        {
            let mut tuner = SearchTuner {
                parameters: vec![],
                ranges: vec![],
            };
            fill_vec!(&mut tuner, $name: $var_type = range($min, $max);
            $($name_1: $var_type_1 = range($min_1, $max_1);)*);
            tuner
        }
    }
}

impl SearchTuner {
    fn param_count(&self) -> usize {
        self.parameters.len()
    }

    fn get_range(&self, param: usize) -> (i16, i16) {
        self.ranges[param]
    }

    fn set_param(&self, param: usize, value: i16) {
        self.parameters[param](value)
    }
}

struct Spsa {
    lr: f32,
    step: f32,
    alpha: f32,
    gamma: f32,
    div: u64,
}

impl Spsa {
    pub fn new(lr: f32, step: f32, target_iter: u64, tuner: SearchTuner) -> Spsa {
        Self {
            lr,
            step,
            alpha: 0.601,
            gamma: 0.102,
            div: target_iter / 10,
        }
    }
}

struct DataPoint {
    board: Board,
    mv: Move,
}

pub fn tune(path: &str) {
    let search_tuner = search_tuner! {
        PAWN: i16 = range(50, 150);
        MINOR: i16 = range(250, 350);
        ROOK: i16 = range(400, 600);
        QUEEN: i16 = range(700, 1100);
    };

    //TODO: Lazy read
    let file = std::fs::read_to_string(path).unwrap();
    let mut data = vec![];
    for line in file.lines() {
        let (board, mv) = line.split_once(" | ").unwrap();
        let board = Board::from_str(board).unwrap();
        let mv = Move::from_str(mv).unwrap();
        data.push(DataPoint { board, mv })
    }

    let time_man = Arc::new(TimeManager::new());
    let mut ab_runner = AbRunner::new(Board::default(), time_man.clone());

    println!("{}", test(&mut ab_runner, time_man, 1000, &data));
}

fn test(
    ab_runner: &mut AbRunner,
    time_manager: Arc<TimeManager>,
    nodes: u64,
    data_points: &[DataPoint],
) -> f32 {
    let mut new_data = String::new();
    let mut loss = 0.0;
    for (index, data_point) in data_points.iter().enumerate().take(1000) {
        ab_runner.new_game();
        ab_runner.set_board(data_point.board.clone());
        time_manager.initiate(&data_point.board, &[TimeManagementInfo::MaxNodes(nodes)]);
        let (mv, eval, _, _) = ab_runner.search::<Run, NoInfo>(1);

        if eval.raw().abs() > 3000 {
            continue;
        }
        
        if mv != data_point.mv {
            /*
            new_data.push_str(&format!(
                "{} | {}\n",
                data_point.board.to_string(),
                data_point.mv.to_string()
            ));
            */
            loss += 1.0;
        }
        
        println!("{} {}", index, loss);
    }
    //std::fs::write("filtered.txt", new_data).unwrap();
    loss
}
