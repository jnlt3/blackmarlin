/*
use std::arch::aarch64;
use std::arch::aarch64::{int16x8_t, int8x16_t};

const UNITS: i16 = 400i16;
const SCALE: i8 = 64;
const SCALE_SHIFT: i32 = 6;
const CHUNKS: usize = 16;

const MIN: [i16; 8] = [0; 8];
const MAX: [i16; 8] = [SCALE as i16; 8];

fn int8x16_from_slice(slice: &[i8]) -> int8x16_t {
    unsafe {
        std::mem::transmute::<[i8; 16], int8x16_t>([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
            slice[8], slice[9], slice[10], slice[11], slice[12], slice[13], slice[14], slice[15],
        ])
    }
}

fn int16x8_from_slice(slice: &[i16]) -> int16x8_t {
    unsafe {
        std::mem::transmute::<[i16; 8], int16x8_t>([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
        ])
    }
}

#[derive(Clone, Debug)]
pub struct IncrementalLayer {
    pub input: usize,
    pub output: usize,
    pub weights: Vec<int8x16_t>,
}

impl IncrementalLayer {
    pub fn with_weights(input: usize, weights: &[f32]) -> Self {
        assert_eq!(0, weights.len() % input);
        let output = weights.len() / input;
        assert_eq!(0, output % CHUNKS);
        let int_weights = weights
            .iter()
            .map(|weight| (*weight * SCALE as f32).round() as i8)
            .collect::<Vec<_>>();
        let packed_weights = int_weights
            .chunks_exact(CHUNKS)
            .map(|chunk| int8x16_from_slice(chunk))
            .collect::<Vec<_>>();
        Self {
            input,
            output,
            weights: packed_weights,
        }
    }

    pub fn incr_ff(&self, input: &[(i8, usize)], output: &mut [int16x8_t]) {
        for (val, index) in input.iter() {
            let start = (index * self.output) / CHUNKS;
            if *val > 0i8 {
                for (out, weight) in output.chunks_exact_mut(2).zip(self.weights[start..].iter()) {
                    unsafe {
                        let weight_low = aarch64::vget_low_s8(*weight);
                        let weight_high = aarch64::vget_high_s8(*weight);
                        out[0] = aarch64::vaddw_s8(out[0], weight_low);
                        out[1] = aarch64::vaddw_s8(out[1], weight_high);
                    }
                }
            } else {
                for (out, weight) in output.chunks_exact_mut(2).zip(self.weights[start..].iter()) {
                    unsafe {
                        let weight_low = aarch64::vget_low_s8(*weight);
                        let weight_high = aarch64::vget_high_s8(*weight);
                        out[0] = aarch64::vsubw_s8(out[0], weight_low);
                        out[1] = aarch64::vsubw_s8(out[1], weight_high);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DenseLayer {
    pub input: usize,
    pub output: usize,
    pub weights: Vec<int8x16_t>,
}

impl DenseLayer {
    pub fn with_weights(input: usize, weights: &[f32]) -> Self {
        assert_eq!(0, weights.len() % input);
        let output = weights.len() / input;
        assert_eq!(0, output % 16);
        let int_weights = weights
            .iter()
            .map(|weight| (*weight * SCALE as f32).round() as i8)
            .collect::<Vec<_>>();
        let packed_weights = int_weights
            .chunks_exact(CHUNKS)
            .map(|chunk| int8x16_from_slice(chunk))
            .collect::<Vec<_>>();
        Self {
            input,
            output,
            weights: packed_weights,
        }
    }

    pub fn ff(&self, input: &[int8x16_t], output: &mut [i16]) {
        for (out, weights) in output
            .iter_mut()
            .zip(self.weights.chunks_exact(self.input / CHUNKS))
        {
            *out = 0;
            unsafe {
                for (input, w) in input.iter().zip(weights.iter()) {
                    let weight_low = aarch64::vget_low_s8(*w);
                    let weight_high = aarch64::vget_high_s8(*w);
                    let input_low = aarch64::vget_low_s8(*input);
                    let input_high = aarch64::vget_high_s8(*input);
                    let mul_low = aarch64::vmull_s8(input_low, weight_low);
                    let mul_high = aarch64::vmull_s8(input_high, weight_high);
                    *out += aarch64::vaddvq_s16(mul_low) + aarch64::vaddvq_s16(mul_high);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct OutputLayer {
    pub input: usize,
    pub output: usize,
    pub weights: Vec<int8x16_t>,
}

impl OutputLayer {
    pub fn with_weights(input: usize, weights: &[f32]) -> Self {
        assert_eq!(0, weights.len() % input);
        let output = weights.len() / input;
        assert_eq!(output, 1);
        let int_weights = weights
            .iter()
            .map(|weight| (*weight * SCALE as f32).round() as i8)
            .collect::<Vec<_>>();
        let packed_weights = int_weights
            .chunks_exact(CHUNKS)
            .map(|chunk| int8x16_from_slice(chunk))
            .collect::<Vec<_>>();
        Self {
            input,
            output,
            weights: packed_weights,
        }
    }

    pub fn ff(&self, input: &[int8x16_t], output: &mut i32) {
        let mut out_buff = 0f32;
        for (val, weights) in input.iter().zip(&self.weights) {
            unsafe {
                let input_low = aarch64::vget_low_s8(*val);
                let input_high = aarch64::vget_high_s8(*val);
                let weights_low = aarch64::vget_low_s8(*weights);
                let weights_high = aarch64::vget_high_s8(*weights);
                let out_low = aarch64::vmull_s8(input_low, weights_low);
                let out_high = aarch64::vmull_s8(input_high, weights_high);
                out_buff += aarch64::vaddvq_s16(aarch64::vaddq_s16(out_low, out_high)) as f32;
            }
        }
        out_buff *= UNITS as f32;
        out_buff /= SCALE as f32;
        out_buff /= SCALE as f32;
        *output = out_buff as i32;
    }
}

pub fn clipped_relu_int16x8(input: &[int16x8_t], output: &mut [int8x16_t]) {
    for (input, clipped) in input.chunks_exact(2).zip(output.iter_mut()) {
        unsafe {
            let input_low = aarch64::vminq_s16(
                aarch64::vmaxq_s16(input[0], int16x8_from_slice(&MIN)),
                int16x8_from_slice(&MAX),
            );
            let input_high = aarch64::vminq_s16(
                aarch64::vmaxq_s16(input[1], int16x8_from_slice(&MIN)),
                int16x8_from_slice(&MAX),
            );
            let clipped_low = aarch64::vreinterpretq_s8_s16(input_low);
            let clipped_high = aarch64::vreinterpretq_s8_s16(input_high);
            *clipped = aarch64::vuzp1q_s8(clipped_low, clipped_high);
        }
    }
}

pub fn scale(input: &[i16], output: &mut [int16x8_t]) {
    for (input, output) in input.chunks_exact(CHUNKS / 2).zip(output) {
        unsafe {
            *output = aarch64::vshrq_n_s16::<SCALE_SHIFT>(int16x8_from_slice(input));
        }
    }
}
*/