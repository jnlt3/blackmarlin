const UNITS: i16 = 170_i16;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = SCALE;

#[derive(Debug, Clone)]
pub struct Psqt<'a, const INPUT: usize, const OUTPUT: usize> {
    weights: &'a [[i32; OUTPUT]; INPUT],
    out: [i32; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Psqt<'a, INPUT, OUTPUT> {
    pub fn new(weights: &'a [[i32; OUTPUT]; INPUT]) -> Self {
        Self {
            weights,
            out: [0_i32; OUTPUT],
        }
    }

    #[inline]
    pub fn incr_ff<const CHANGE: i32>(&mut self, index: usize) {
        for (out, &weight) in self.out.iter_mut().zip(&self.weights[index]) {
            *out += weight * CHANGE;
        }
    }

    pub fn get(&self) -> &[i32; OUTPUT] {
        &self.out
    }
}

#[derive(Debug, Clone)]
pub struct Incremental<'a, const INPUT: usize, const OUTPUT: usize> {
    weights: &'a [[i8; OUTPUT]; INPUT],
    out: [i16; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Incremental<'a, INPUT, OUTPUT> {
    pub fn new(weights: &'a [[i8; OUTPUT]; INPUT], bias: [i16; OUTPUT]) -> Self {
        Self { weights, out: bias }
    }

    #[inline]
    pub fn incr_ff<const CHANGE: i16>(&mut self, index: usize) {
        for (out, &weight) in self.out.iter_mut().zip(&self.weights[index]) {
            *out += weight as i16 * CHANGE;
        }
    }

    pub fn get(&self) -> &[i16; OUTPUT] {
        &self.out
    }
}

#[derive(Debug, Clone)]
pub struct Dense<'a, const INPUT: usize, const OUTPUT: usize> {
    weights: &'a [[i8; INPUT]; OUTPUT],
    bias: [i32; OUTPUT],
}

impl<'a, const INPUT: usize, const OUTPUT: usize> Dense<'a, INPUT, OUTPUT> {
    pub fn new(weights: &'a [[i8; INPUT]; OUTPUT], bias: [i16; OUTPUT]) -> Self {
        Self {
            weights,
            bias: i16_to_i32(bias),
        }
    }

    #[inline]
    pub fn ff_sym(
        &self,
        w_inputs: &[u8; INPUT],
        b_inputs: &[u8; INPUT],
        bucket: usize,
    ) -> [i32; OUTPUT] {
        let mut out = self.bias;
        for ((&w_input, &b_input), weight) in w_inputs
            .iter()
            .zip(b_inputs.iter())
            .zip(&self.weights[bucket])
        {
            out[bucket] += *weight as i32 * (w_input as i32 - b_input as i32) / 2;
        }
        out
    }
}

#[inline]
pub fn out(x: i32) -> i16 {
    (x as i64 * UNITS as i64 / (SCALE * SCALE) as i64) as i16
}

#[inline]
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
pub fn clipped_relu<const N: usize>(array: [i16; N]) -> [u8; N] {
    let mut out = [0_u8; N];
    for (&x, clipped) in array.iter().zip(out.iter_mut()) {
        *clipped = x.max(MIN).min(MAX) as u8;
    }
    out
}
