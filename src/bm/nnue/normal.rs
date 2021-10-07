use serde::{Deserialize, Serialize};

const UNITS: i16 = 300_i16;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = SCALE;

#[derive(Serialize, Deserialize)]
pub struct NonConstWeights(pub Vec<Vec<f32>>);

#[derive(Debug, Clone)]
pub struct Weights<const INPUT: usize, const OUTPUT: usize>(Box<[[i8; OUTPUT]; INPUT]>);

impl<const INPUT: usize, const OUTPUT: usize> Into<Weights<INPUT, OUTPUT>> for NonConstWeights {
    fn into(self) -> Weights<INPUT, OUTPUT> {
        assert_eq!(self.0.len(), OUTPUT);
        self.0.iter().for_each(|vec| assert_eq!(vec.len(), INPUT));
        let mut weights = Box::new([[0_i8; OUTPUT]; INPUT]);
        for i in 0..INPUT {
            for j in 0..OUTPUT {
                weights[i][j] = (self.0[j][i] * SCALE as f32).round() as i8;
            }
        }
        Weights(weights)
    }
}

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Weights<INPUT, OUTPUT>,
    out: [i16; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
    pub fn new(weights: Weights<INPUT, OUTPUT>) -> Self {
        Self {
            weights,
            out: [0_i16; OUTPUT],
        }
    }

    #[inline]
    pub fn incr_ff(&mut self, index: usize, change: i8) {
        for (out, &weight) in self.out.iter_mut().zip(&self.weights.0[index]) {
            *out += (weight * change) as i16;
        }
    }

    pub fn get(&self) -> &[i16; OUTPUT] {
        &self.out
    }
}

#[derive(Debug, Clone)]
pub struct Dense<const INPUT: usize, const OUTPUT: usize> {
    weights: Weights<INPUT, OUTPUT>,
}

impl<const INPUT: usize, const OUTPUT: usize> Dense<INPUT, OUTPUT> {
    pub fn new(weights: Weights<INPUT, OUTPUT>) -> Self {
        Self { weights }
    }

    #[inline]
    pub fn ff(&self, inputs: &[i8; INPUT]) -> [i16; OUTPUT] {
        let mut out = [0_i16; OUTPUT];
        for (&input, weights) in inputs.iter().zip(&*self.weights.0) {
            for (out, &weight) in out.iter_mut().zip(weights) {
                *out += weight as i16 * input as i16;
            }
        }
        out
    }

    #[inline]
    pub fn ff_sym(&self, w_inputs: &[i8; INPUT], b_inputs: &[i8; INPUT]) -> [i16; OUTPUT] {
        let mut out = [0_i16; OUTPUT];
        for ((&w_input, &b_input), weights) in
            w_inputs.iter().zip(b_inputs.iter()).zip(&*self.weights.0)
        {
            for (out, &weight) in out.iter_mut().zip(weights) {
                *out += weight as i16 * (w_input as i16 - b_input as i16) / 2;
            }
        }
        out
    }
}

pub fn out(x: i16) -> i16 {
    (x as f32 * UNITS as f32 / (SCALE * SCALE) as f32) as i16
}

#[inline]
pub fn scale<const N: usize>(array: &mut [i16; N]) {
    for x in array {
        *x /= SCALE;
    }
}

#[inline]
pub fn clipped_relu<const N: usize>(array: [i16; N]) -> [i8; N] {
    let mut out = [0_i8; N];
    for (&x, clipped) in array.iter().zip(out.iter_mut()) {
        *clipped = x.max(MIN).min(MAX) as i8;
    }
    out
}
