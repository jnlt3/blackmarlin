use std::sync::Arc;

const UNITS: i16 = 400_i16;
const FT_SCALE: i16 = 255;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = FT_SCALE;
const SHIFT: i16 = 8;

const INTERMEDIATE_DIV: i16 = 16;
const INTERMEDIATE_SHIFT: i16 = 4;

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<[[i16; OUTPUT]; INPUT]>,
    out: [i16; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
    pub fn new(weights: Arc<[[i16; OUTPUT]; INPUT]>, bias: [i16; OUTPUT]) -> Self {
        Self { weights, out: bias }
    }

    pub fn reset(&mut self, out: [i16; OUTPUT]) {
        self.out = out;
    }

    #[inline]
    pub fn incr_ff<const CHANGE: i16>(&mut self, index: usize) {
        for (out, &weight) in self.out.iter_mut().zip(&self.weights[index]) {
            *out += weight * CHANGE;
        }
    }

    pub fn get(&self) -> &[i16; OUTPUT] {
        &self.out
    }
}

#[derive(Debug, Clone)]
pub struct Dense<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<[[i16; INPUT]; OUTPUT]>,
    bias: [i16; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Dense<INPUT, OUTPUT> {
    pub fn new(weights: Arc<[[i16; INPUT]; OUTPUT]>, bias: [i16; OUTPUT]) -> Self {
        let mut scaled_bias = [0; OUTPUT];
        scaled_bias
            .iter_mut()
            .zip(bias)
            .for_each(|(scaled, value)| *scaled = value >> INTERMEDIATE_SHIFT);
        Self {
            weights,
            bias: scaled_bias,
        }
    }

    pub fn ff(&self, inputs: &[u8; INPUT]) -> [i16; OUTPUT] {
        let mut out = self.bias;
        for (out, weights) in out.iter_mut().zip(&*self.weights) {
            for (&input, &weight) in inputs.iter().zip(weights.iter()) {
                *out += (weight * input as i16) >> INTERMEDIATE_SHIFT;
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct Out<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<[[i16; INPUT]; OUTPUT]>,
    bias: [i32; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Out<INPUT, OUTPUT> {
    pub fn new(weights: Arc<[[i16; INPUT]; OUTPUT]>, bias: [i32; OUTPUT]) -> Self {
        Self { weights, bias }
    }

    pub fn ff(&self, inputs: &[u8; INPUT]) -> [i32; OUTPUT] {
        let mut out = self.bias;
        for (out, weights) in out.iter_mut().zip(&*self.weights) {
            for (&input, &weight) in inputs.iter().zip(weights.iter()) {
                *out += weight as i32 * input as i32;
            }
        }
        out
    }
}

#[inline]
pub fn out(x: i32) -> i16 {
    (x as f32 * UNITS as f32 / (FT_SCALE as f32 * SCALE as f32)) as i16
}

#[inline]
pub fn sq_clipped_relu<const N: usize>(array: [i16; N], out: &mut [u8]) {
    for (&x, clipped) in array.iter().zip(out.iter_mut()) {
        let tmp = x.max(MIN).min(MAX) as u16;
        *clipped = ((tmp * tmp) >> SHIFT) as u8;
    }
}

#[inline]
pub fn scaled_clipped_relu<const N: usize>(array: [i16; N], out: &mut [u8]) {
    for (&x, clipped) in array.iter().zip(out.iter_mut()) {
        *clipped = (x / (SCALE / INTERMEDIATE_DIV)).max(MIN).min(MAX) as u8;
    }
}
