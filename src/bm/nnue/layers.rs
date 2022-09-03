use std::sync::Arc;

use cfg_if::cfg_if;

const UNITS: i16 = 400_i16;
const FT_SCALE: i16 = 255;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = FT_SCALE;
const SHIFT: i16 = 8;

#[derive(Debug, Copy, Clone)]
#[repr(align(64))]
pub struct Align<T>(pub T);

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<Align<[[i16; OUTPUT]; INPUT]>>,
    out: Align<[i16; OUTPUT]>,
}

impl<const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
    pub fn new(weights: Arc<Align<[[i16; OUTPUT]; INPUT]>>, bias: Align<[i16; OUTPUT]>) -> Self {
        Self { weights, out: bias }
    }

    pub fn reset(&mut self, out: [i16; OUTPUT]) {
        self.out.0 = out;
    }

    #[inline]
    pub fn incr_ff<const CHANGE: i16>(&mut self, index: usize) {
        for (out, &weight) in self.out.0.iter_mut().zip(&self.weights.0[index]) {
            *out += weight * CHANGE;
        }
    }

    pub fn get(&self) -> &[i16; OUTPUT] {
        &self.out.0
    }
}

#[derive(Debug, Clone)]
pub struct Dense<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<Align<[[i8; INPUT]; OUTPUT]>>,
    bias: Align<[i32; OUTPUT]>,
}

impl<const INPUT: usize, const OUTPUT: usize> Dense<INPUT, OUTPUT> {
    pub fn new(weights: Arc<Align<[[i8; INPUT]; OUTPUT]>>, bias: Align<[i32; OUTPUT]>) -> Self {
        Self { weights, bias }
    }

    #[inline]
    pub fn ff(&self, inputs: &[u8; INPUT], bucket: usize) -> i32 {
        let mut out = self.bias.0[bucket];
        for (&input, &weight) in inputs.iter().zip(self.weights.0[bucket].iter()) {
            out += weight as i32 * input as i32;
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
