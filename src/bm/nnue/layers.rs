use std::sync::Arc;

const UNITS: i16 = 400_i16;
const FT_SCALE: i16 = 255;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = FT_SCALE;

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<[[i16; OUTPUT]; INPUT]>,
    out: [i16; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
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
    weights: Arc<[[i8; OUTPUT]; INPUT]>,
    bias: [i32; OUTPUT],
}

impl<const INPUT: usize, const OUTPUT: usize> Dense<INPUT, OUTPUT> {
    pub fn new(weights: Arc<[[i8; OUTPUT]; INPUT]>, bias: [i32; OUTPUT]) -> Self {
        Self { weights, bias }
    }

    #[inline]
    pub fn ff(&self, inputs: &[u8; INPUT], bucket: usize) -> [i32; OUTPUT] {
        let mut out = self.bias;
        for (&input, weights) in inputs.iter().zip(&*self.weights) {
            for (out, &weight) in out[bucket..bucket + 1]
                .iter_mut()
                .zip(weights[bucket..bucket + 1].iter())
            {
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
pub fn mul_clipped_relu<const N: usize>(array: [i16; N], out: &mut [u8]) {
    for ((&x_0, &x_1), clipped) in array.iter().zip(&array[N / 2..]).zip(out) {
        *clipped = (x_0.max(MIN).min(MAX) as u16 * x_1.max(MIN).min(MAX) as u16 / MAX as u16) as u8;
    }
}
