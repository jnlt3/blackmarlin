const UNITS: i16 = 300_i16;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = SCALE;

#[derive(Debug, Clone)]
pub struct Weights<'a, const INPUT: usize, const OUTPUT: usize>(&'a [[i8; OUTPUT]; INPUT]);

#[derive(Debug, Clone)]
pub struct Incremental<'a, const INPUT: usize, const OUTPUT: usize> {
    weights: Weights<'a, INPUT, OUTPUT>,
    out: [i16; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Incremental<'a, INPUT, OUTPUT> {
    pub fn new(weights: &'a [[i8; OUTPUT]; INPUT], bias: [i16; OUTPUT]) -> Self {
        Self {
            weights: Weights(weights),
            out: bias,
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
pub struct Dense<'a, const INPUT: usize, const OUTPUT: usize> {
    weights: Weights<'a, INPUT, OUTPUT>,
    bias: [i32; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Dense<'a, INPUT, OUTPUT> {
    pub fn new(weights: &'a [[i8; OUTPUT]; INPUT], bias: [i16; OUTPUT]) -> Self {
        Self {
            weights: Weights(weights),
            bias: i16_to_i32(bias),
        }
    }

    #[inline]
    pub fn ff_sym(&self, w_inputs: &[i8; INPUT], b_inputs: &[i8; INPUT]) -> [i32; OUTPUT] {
        let mut out = self.bias;
        for ((&w_input, &b_input), weights) in
            w_inputs.iter().zip(b_inputs.iter()).zip(&*self.weights.0)
        {
            for (out, &weight) in out.iter_mut().zip(weights) {
                *out += weight as i32 * (w_input as i32 - b_input as i32) / 2;
            }
        }
        out
    }
}

pub fn out(x: i16) -> i16 {
    (x as f32 * UNITS as f32 / (SCALE * SCALE) as f32) as i16
}

const fn i16_to_i32<const N: usize>(array: [i16; N]) -> [i32; N] {
    let mut out = [0_i32; N];
    let mut index = 0;
    while index < N {
        out[index] = array[index] as i32;
        index += 1;
    }
    out
}

#[inline]
pub fn clipped_relu<const N: usize>(array: [i16; N]) -> [i8; N] {
    let mut out = [0_i8; N];
    for (&x, clipped) in array.iter().zip(out.iter_mut()) {
        *clipped = x.max(MIN).min(MAX) as i8;
    }
    out
}