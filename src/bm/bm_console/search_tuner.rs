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
}

pub fn create() {
    let search_tuner = search_tuner! {
        PAWN: i16 = range(50, 150);
        MINOR: i16 = range(250, 350);
        ROOK: i16 = range(400, 600);
        QUEEN: i16 = range(700, 1100);
    };
    
}
