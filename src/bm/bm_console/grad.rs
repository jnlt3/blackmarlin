use arrayvec::ArrayVec;
use rand::Rng;

use crate::bm::bm_eval::evaluator::{BbPair, RanksPair};
use crate::bm::bm_eval::evaluator::{EvalTrace, Indices};
use std::ops::AddAssign;
use std::ops::DivAssign;
use std::ops::MulAssign;
use std::ops::SubAssign;

use super::gen_fen::DataPoint;

trait Stringify {
    fn string(&self) -> String;
}

trait Sqrt {
    fn sqrt(&self) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct T(f32, f32);

impl T {
    fn convert(&self, phase: f32) -> f32 {
        (self.0 * phase + self.1 * (24.0 - phase)) / 24.0
    }
}

impl Sqrt for T {
    fn sqrt(&self) -> Self {
        T(self.0.sqrt(), self.1.sqrt())
    }
}

impl Stringify for T {
    fn string(&self) -> String {
        format!(
            "TaperedEval({}, {})",
            self.0.round() as i16,
            self.1.round() as i16
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct RankTable([T; 8]);

impl Sqrt for RankTable {
    fn sqrt(&self) -> Self {
        let mut new_table = [T::default(); 8];
        for (rank, new_rank) in self.0.iter().zip(new_table.iter_mut()) {
            *new_rank = rank.sqrt();
        }
        RankTable(new_table)
    }
}

impl Stringify for RankTable {
    fn string(&self) -> String {
        let mut string = "[".to_string();
        for eval in &self.0 {
            string += &(eval.string().to_string() + ",");
        }
        string += "]";
        string
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct SquareTable([[T; 8]; 8]);

impl Sqrt for SquareTable {
    fn sqrt(&self) -> Self {
        let mut new_table = [[T::default(); 8]; 8];
        for (rank, new_rank) in self.0.iter().zip(new_table.iter_mut()) {
            for (sq, new_sq) in rank.iter().zip(new_rank.iter_mut()) {
                *new_sq = sq.sqrt();
            }
        }
        SquareTable(new_table)
    }
}

impl Stringify for SquareTable {
    fn string(&self) -> String {
        let mut string = "[".to_string();
        for eval in &self.0 {
            string += "[";
            for eval in eval {
                string += &(eval.string().to_string() + ",");
            }
            string += "],\n";
        }
        string += "]";
        string
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexTable<const SIZE: usize>(ArrayVec<T, SIZE>);

impl<const SIZE: usize> Default for IndexTable<SIZE> {
    fn default() -> Self {
        Self(ArrayVec::from([T::default(); SIZE]))
    }
}

impl<const SIZE: usize> Sqrt for IndexTable<SIZE> {
    fn sqrt(&self) -> Self {
        let mut new = ArrayVec::<T, SIZE>::new();
        for i in &self.0 {
            new.push(i.sqrt());
        }
        IndexTable(new)
    }
}

impl<const SIZE: usize> Stringify for IndexTable<SIZE> {
    fn string(&self) -> String {
        let mut string = "[".to_string();
        for element in &self.0 {
            string += &format!("{}, ", element.string());
        }
        string += "]";
        string
    }
}

impl<const CAP: usize, const SIZE: usize> std::ops::Mul<Indices<CAP, SIZE>> for IndexTable<SIZE> {
    type Output = T;

    fn mul(self, rhs: Indices<CAP, SIZE>) -> Self::Output {
        let mut out = T::default();
        for &index in &rhs.0 {
            let index = index as usize;
            out += self.0[index];
        }
        for &index in &rhs.1 {
            let index = index as usize;
            out -= self.0[index];
        }
        out
    }
}

impl<const CAP: usize, const SIZE: usize> std::ops::Mul<T> for Indices<CAP, SIZE> {
    type Output = IndexTable<SIZE>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut out = ArrayVec::from([T(0.0, 0.0); SIZE]);
        for &index in &self.0 {
            let index = index as usize;
            out[index] += rhs;
        }
        for &index in &self.1 {
            let index = index as usize;
            out[index] -= rhs;
        }
        IndexTable::<SIZE>(out)
    }
}

macro_rules! impl_op {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait<f32> for T {
            type Output = T;

            fn $op(self, rhs: f32) -> Self::Output {
                T(self.0.$op(rhs), self.1.$op(rhs))
            }
        }

        impl std::ops::$trait<i16> for T {
            type Output = T;

            fn $op(self, rhs: i16) -> Self::Output {
                let rhs = rhs as f32;
                T(self.0.$op(rhs), self.1.$op(rhs))
            }
        }

        impl std::ops::$trait<T> for i16 {
            type Output = T;

            fn $op(self, rhs: T) -> Self::Output {
                T((self as f32).$op(rhs.0), (self as f32).$op(rhs.1))
            }
        }

        impl std::ops::$trait for T {
            type Output = T;

            fn $op(self, rhs: Self) -> Self::Output {
                T(self.0.$op(rhs.0), self.1.$op(rhs.1))
            }
        }

        impl std::ops::$trait for RankTable {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                let mut out = RankTable::default();
                for i in 0..8 {
                    out.0[i] = self.0[i].$op(rhs.0[i]);
                }
                out
            }
        }

        impl std::ops::$trait for SquareTable {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                let mut out = SquareTable::default();
                for r in 0..8 {
                    for f in 0..8 {
                        out.0[r][f] = self.0[r][f].$op(rhs.0[r][f]);
                    }
                }
                out
            }
        }

        impl std::ops::$trait<f32> for RankTable {
            type Output = Self;
            fn $op(self, rhs: f32) -> Self::Output {
                let mut out = RankTable::default();
                for i in 0..8 {
                    out.0[i] = self.0[i].$op(rhs);
                }
                out
            }
        }

        impl std::ops::$trait<f32> for SquareTable {
            type Output = Self;
            fn $op(self, rhs: f32) -> Self::Output {
                let mut out = SquareTable::default();
                for r in 0..8 {
                    for f in 0..8 {
                        out.0[r][f] = self.0[r][f].$op(rhs);
                    }
                }
                out
            }
        }

        impl std::ops::$trait<T> for RankTable {
            type Output = Self;
            fn $op(self, rhs: T) -> Self::Output {
                let mut out = RankTable::default();
                for i in 0..8 {
                    out.0[i] = self.0[i].$op(rhs);
                }
                out
            }
        }

        impl std::ops::$trait<T> for SquareTable {
            type Output = Self;
            fn $op(self, rhs: T) -> Self::Output {
                let mut out = SquareTable::default();
                for r in 0..8 {
                    for f in 0..8 {
                        out.0[r][f] = self.0[r][f].$op(rhs);
                    }
                }
                out
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($trait:ident, $op:ident) => {
        impl std::ops::$trait for T {
            fn $op(&mut self, rhs: Self) {
                self.0.$op(rhs.0);
                self.1.$op(rhs.1);
            }
        }

        impl std::ops::$trait<f32> for T {
            fn $op(&mut self, rhs: f32) {
                self.0.$op(rhs);
                self.1.$op(rhs);
            }
        }

        impl std::ops::$trait<i16> for T {
            fn $op(&mut self, rhs: i16) {
                let rhs = rhs as f32;
                self.0.$op(rhs);
                self.1.$op(rhs);
            }
        }

        impl std::ops::$trait for RankTable {
            fn $op(&mut self, rhs: Self) {
                for i in 0..8 {
                    self.0[i].$op(rhs.0[i]);
                }
            }
        }

        impl std::ops::$trait for SquareTable {
            fn $op(&mut self, rhs: Self) {
                for r in 0..8 {
                    for f in 0..8 {
                        self.0[r][f].$op(rhs.0[r][f]);
                    }
                }
            }
        }

        impl<const SIZE: usize> std::ops::$trait for IndexTable<SIZE> {
            fn $op(&mut self, rhs: Self) {
                for r in 0..SIZE {
                    self.0[r].$op(rhs.0[r]);
                }
            }
        }

        impl std::ops::$trait<f32> for RankTable {
            fn $op(&mut self, rhs: f32) {
                for i in 0..8 {
                    self.0[i].$op(rhs);
                }
            }
        }

        impl std::ops::$trait<f32> for SquareTable {
            fn $op(&mut self, rhs: f32) {
                for r in 0..8 {
                    for f in 0..8 {
                        self.0[r][f].$op(rhs);
                    }
                }
            }
        }

        impl<const SIZE: usize> std::ops::$trait<f32> for IndexTable<SIZE> {
            fn $op(&mut self, rhs: f32) {
                for r in 0..SIZE {
                    self.0[r].$op(rhs);
                }
            }
        }

        impl std::ops::$trait<T> for RankTable {
            fn $op(&mut self, rhs: T) {
                for i in 0..8 {
                    self.0[i].$op(rhs);
                }
            }
        }

        impl std::ops::$trait<T> for SquareTable {
            fn $op(&mut self, rhs: T) {
                for r in 0..8 {
                    for f in 0..8 {
                        self.0[r][f].$op(rhs);
                    }
                }
            }
        }

        impl<const SIZE: usize> std::ops::$trait<T> for IndexTable<SIZE> {
            fn $op(&mut self, rhs: T) {
                for r in 0..SIZE {
                    self.0[r].$op(rhs);
                }
            }
        }
    };
}

impl std::ops::Mul<RanksPair> for RankTable {
    type Output = T;
    fn mul(self, rhs: RanksPair) -> Self::Output {
        let mut eval = T::default();
        for sq in rhs.0 {
            let rank = sq.get_rank().to_index();
            eval += self.0[rank];
        }
        for sq in rhs.1 {
            let rank = sq.get_rank().to_index();
            eval -= self.0[rank];
        }
        eval
    }
}

impl std::ops::Mul<BbPair> for SquareTable {
    type Output = T;
    fn mul(self, rhs: BbPair) -> Self::Output {
        let mut eval = T::default();
        for sq in rhs.0 {
            let rank = sq.get_rank().to_index();
            let file = sq.get_file().to_index();
            eval += self.0[rank][file];
        }
        for sq in rhs.1 {
            let rank = sq.get_rank().to_index();
            let file = sq.get_file().to_index();
            eval -= self.0[rank][file];
        }
        eval
    }
}

macro_rules! impl_pairs {
    ($ty:ty) => {
        impl std::ops::Mul<RanksPair> for $ty {
            type Output = RankTable;

            fn mul(self, rhs: RanksPair) -> Self::Output {
                let mut table = RankTable::default();
                for sq in rhs.0 {
                    let rank = sq.get_rank().to_index();
                    table.0[rank] += self;
                }
                for sq in rhs.1 {
                    let rank = sq.get_rank().to_index();
                    table.0[rank] -= self;
                }
                table
            }
        }

        impl std::ops::Mul<$ty> for RanksPair {
            type Output = RankTable;

            fn mul(self, rhs: $ty) -> Self::Output {
                rhs * self
            }
        }

        impl std::ops::Mul<BbPair> for $ty {
            type Output = SquareTable;

            fn mul(self, rhs: BbPair) -> Self::Output {
                let mut table = SquareTable::default();
                for sq in rhs.0 {
                    let rank = sq.get_rank().to_index();
                    let file = sq.get_file().to_index();
                    table.0[rank][file] += self;
                }
                for sq in rhs.1 {
                    let rank = sq.get_rank().to_index();
                    let file = sq.get_file().to_index();
                    table.0[rank][file] -= self;
                }
                table
            }
        }

        impl std::ops::Mul<$ty> for BbPair {
            type Output = SquareTable;

            fn mul(self, rhs: $ty) -> Self::Output {
                rhs * self
            }
        }
    };
}

impl_pairs!(f32);
impl_pairs!(T);

impl_op!(Add, add);
impl_op!(Mul, mul);
impl_op!(Sub, sub);
impl_op!(Div, div);
impl_op_assign!(AddAssign, add_assign);
impl_op_assign!(MulAssign, mul_assign);
impl_op_assign!(SubAssign, sub_assign);
impl_op_assign!(DivAssign, div_assign);

macro_rules! set_grad {
    ($weights: expr, $trace: expr, $grad: expr, $element: ident: $ty: ty) => {
        $grad.$element += $trace.$element.clone() * $weights;
    };
    ($weights: expr, $trace: expr, $grad: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        {
            $grad.$element += $trace.$element.clone() * $weights;
            set_grad!($weights, $trace, $grad, $($elements: $ty),*);
        }
    }
}

macro_rules! apply_func {
    ($op: ident, $first: expr, $element: ident: $ty: ty) => {
        $first.$element.$op();
    };
    ($op: ident, $first: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        $first.$element.$op();
        apply_func!($op, $first, $($elements: $tys),*);
    }
}

macro_rules! apply_op {
    ($op: ident, $first: expr, $second: expr, $element: ident: $ty: ty) => {
        $first.$element.$op($second.$element.clone());
    };
    ($op: ident, $first: expr, $second: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        $first.$element.$op($second.$element.clone());
        apply_op!($op, $first, $second, $($elements: $tys),*);
    }
}

macro_rules! apply_op_f32 {
    ($op: ident, $first: expr, $second: expr, $element: ident: $ty: ty) => {
        $first.$element.$op($second);
    };
    ($op: ident, $first: expr, $second: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        $first.$element.$op($second);
        apply_op_f32!($op, $first, $second, $($elements: $tys),*);
    }
}

macro_rules! apply_weights {
    ($weights: expr, $eval_trace: expr, $element: ident: $ty: ty) => {
        ($weights.$element.clone() * $eval_trace.$element.clone())
    };
    ($weights: expr, $eval_trace: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        $weights.$element.clone() * $eval_trace.$element.clone() + apply_weights!($weights, $eval_trace, $($elements: $tys),*)
    }
}

macro_rules! get_fields {
    ($obj: expr, $element: ident: $ty: ty) => {
        format!("pub const {}: {} = {};", stringify!($element).to_ascii_uppercase(), stringify!($ty), $obj.$element.string());
    };
    ($obj: expr, $element: ident: $ty: ty, $($elements: ident: $tys: ty),*) => {
        format!("pub const {}: {} = {};\n {}", stringify!($element).to_ascii_uppercase(), stringify!($ty), $obj.$element.string(), &get_fields!($obj, $($elements: $tys),*));
    }
}

macro_rules! params {
    ($($element: ident: $ty: ty),*) => {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Grad { $($element: $ty),*}

        impl Grad {
            fn apply(&self, weights: &mut Weights) {
                apply_op!(sub_assign, weights, &self, $($element: $ty),*);
            }
        }

        impl Sqrt for Grad {
            fn sqrt(&self) -> Self {
                let mut cloned = self.clone();
                apply_func!(sqrt, &mut cloned, $($element: $ty),*);
                cloned
            }
        }

        macro_rules! impl_grad_op_assign {
            ($assign: ident, $normal: ident, $assign_op: ident, $normal_op: ident) => {
                impl std::ops::$assign for Grad {
                    fn $assign_op(&mut self, rhs: Self)  {
                        apply_op!($assign_op, self, rhs, $($element: $ty),*);
                    }
                }
                impl std::ops::$assign<f32> for Grad {
                    fn $assign_op(&mut self, rhs: f32)  {
                        apply_op_f32!($assign_op, self, rhs, $($element: $ty),*);
                    }
                }
                impl std::ops::$normal for Grad {
                    type Output = Grad;

                    fn $normal_op(self, rhs: Self) -> Self::Output {
                        let mut clone = self.clone();
                        apply_op!($assign_op, &mut clone, rhs, $($element: $ty),*);
                        clone
                    }
                }
                impl std::ops::$normal<f32> for Grad {
                    type Output = Grad;

                    fn $normal_op(self, rhs: f32) -> Self::Output {
                        let mut clone = self.clone();
                        apply_op_f32!($assign_op, &mut clone, rhs, $($element: $ty),*);
                        clone
                    }
                }
            }
        }

        impl_grad_op_assign!(AddAssign, Add, add_assign, add);
        impl_grad_op_assign!(SubAssign, Sub, sub_assign, sub);
        impl_grad_op_assign!(MulAssign, Mul, mul_assign, mul);
        impl_grad_op_assign!(DivAssign, Div, div_assign, div);

        #[derive(Debug, Clone, PartialEq, Default)]
        struct Weights {
            $($element: $ty),*
        }
        impl Weights {
            fn apply(&self, trace: &EvalTrace) -> f32 {
                let eval: T = apply_weights!(&self, trace, $($element: $ty),*);
                eval.convert(trace.phase as f32)
            }

            fn print(&self) {
                println!("{}", get_fields!(&self, $($element: $ty),*))
            }
        }
    };
}

macro_rules! optimizer {
    {$($element: ident: $ty: ty),*,} => {
        optimizer!($($element: $ty),*)
    };
    {$($element: ident: $ty: ty),*} => {
        params!($($element: $ty),*);

        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct Optimizer {
            weights: Weights,

            grad: Grad,
            cache: Grad,

            factor: f32,

            lr: f32,
            beta: f32,
        }

        impl Optimizer {

            fn new(lr: f32, beta: f32, factor: f32) -> Self {
                Self {
                    factor,
                    lr,
                    beta,
                    ..Default::default()
                }
            }

            fn error(&self, data_points: &[DataPoint]) -> f32 {
                let mut err = 0.0;
                let mut sum = 0.0;
                for data_point in data_points {
                    let pred = sigmoid(self.feed_forward(&data_point.trace), self.factor);
                    let diff = data_point.result as f32 - pred;

                    err += data_point.weight as f32 * diff * diff;
                    sum += data_point.weight as f32;
                }
                err / sum
            }

            fn feed_forward(&self, trace: &EvalTrace) -> f32 {
                self.weights.apply(trace)
            }

            fn back_prop(&mut self, trace: &EvalTrace, result: f32, weight: f32) {
                let pred = self.feed_forward(trace);
                let pred_wdl = sigmoid(pred, self.factor);

                let grad = pred_wdl - result;
                let sigmoid_grad = weight * grad * pred_wdl * (1.0 - pred_wdl) * self.lr * self.factor;

                let mg_effect = trace.phase as f32 / 24.0;
                let eg_effect = 1.0 - mg_effect;

                let mut gradient = Grad::default();

                let weights = T(mg_effect * sigmoid_grad, eg_effect * sigmoid_grad);
                set_grad!(weights, trace, gradient, $($element: $ty),*);
                self.grad += gradient;
            }

            fn apply(&mut self) {
                self.cache = self.cache.clone() * self.beta + self.grad.clone() * self.grad.clone() * (1.0 - self.beta);
                ((self.grad.clone() / (self.cache.sqrt() + 1e-8)) * self.lr).apply(&mut self.weights);
                self.grad *= 0.0;
            }
        }
    }
}

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + (-x * k).exp())
}

const PRINT_ITERS: usize = 10000;
const ITERS: usize = PRINT_ITERS * 1000;
const BATCH_SIZE: usize = 256;

pub fn tune(data_points: &[DataPoint]) {
    println!("position count: {}", data_points.len());

    optimizer! {
        tempo: T,
        doubled: T,
        isolated: T,
        chained: T,
        threat: T,
        bishop_pair: T,
        phalanx: T,
        passed_table: RankTable,

        knight_mobility: IndexTable<9>,
        bishop_mobility: IndexTable<14>,
        rook_mobility: IndexTable<15>,
        queen_mobility: IndexTable<28>,

        knight_attack_cnt: T,
        bishop_attack_cnt: T,
        rook_attack_cnt: T,
        queen_attack_cnt: T,

        pawn_cnt: T,
        knight_cnt: T,
        bishop_cnt: T,
        rook_cnt: T,
        queen_cnt: T,

        pawns: SquareTable,
        knights: SquareTable,
        bishops: SquareTable,
        rooks: SquareTable,
        queens: SquareTable,
        kings: SquareTable,
    }

    let mut optim = Box::new(Optimizer::new(0.001, 0.999, 0.0056));
    optim.weights.print();
    println!("err: {}", optim.error(data_points));


    for _ in 0..ITERS / PRINT_ITERS {
        for _ in 0..PRINT_ITERS {
            for _ in 0..BATCH_SIZE {
                let index: usize = rand::thread_rng().gen_range(0..data_points.len());
                let data_point = &data_points[index];
                optim.back_prop(
                    &data_point.trace,
                    data_point.result as f32,
                    data_point.weight as f32,
                );
            }
            optim.apply();
        }
        optim.weights.print();
        println!("err: {}", optim.error(data_points));
        println!("{}", optim.factor);
    }
}
