use std::{ops::Range, sync::Arc};

use cfg_if::cfg_if;

const UNITS: i16 = 400_i16;
const FT_SCALE: i16 = 255;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = FT_SCALE;
const SHIFT: i16 = 8;

#[derive(Debug, Copy, Clone)]
#[repr(C, align(64))]
pub struct Align<T>(pub T);

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<Align<[[i16; OUTPUT]; INPUT]>>,
}

impl<const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
    pub fn new(weights: Arc<Align<[[i16; OUTPUT]; INPUT]>>) -> Self {
        Self { weights }
    }

    pub fn update_features(
        &mut self,
        src: &Align<[i16; OUTPUT]>,
        out: &mut Align<[i16; OUTPUT]>,
        added_features: &[usize],
        removed_features: &[usize],
    ) {
        cfg_if! {
            if #[cfg(target_feature = "avx2")] {
                const CHUNKS: usize = 256;
            } else {
                const CHUNKS: usize = 128;
            }
        }
        for start in 0..(OUTPUT + CHUNKS - 1) / CHUNKS {
            let range = start * CHUNKS..(start * CHUNKS + CHUNKS).min(OUTPUT);
            let mut out_reg = [0; CHUNKS];
            out_reg[..range.len()].copy_from_slice(&src.0[range.clone()]);
            self.update_chunk::<1>(added_features, &mut out_reg, range.clone());
            self.update_chunk::<-1>(removed_features, &mut out_reg, range.clone());
            out.0[range.clone()].copy_from_slice(&out_reg[..range.len()]);
        }
    }

    fn update_chunk<const SIGN: i16>(
        &self,
        feature_indices: &[usize],
        reg: &mut [i16],
        chunk: Range<usize>,
    ) {
        for &index in feature_indices {
            for (out, &weight) in reg.iter_mut().zip(&self.weights.0[index][chunk.clone()]) {
                *out += weight * SIGN;
            }
        }
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

    pub fn feed_forward(&self, inputs: &Align<[u8; INPUT]>, bucket: usize) -> i32 {
        let mut out = self.bias.0[bucket];

        #[cfg(target_feature = "avx2")]
        {
            use std::arch::x86_64::*;
            const VEC_SIZE: usize = std::mem::size_of::<__m256i>() / std::mem::size_of::<u8>();

            // SAFETY: Only enabled on AVX2
            if INPUT % VEC_SIZE == 0 {
                unsafe {
                    let weights = &self.weights.0[bucket];
                    let ones = _mm256_set1_epi16(1);
                    let mut sum = _mm256_setzero_si256();
                    for (inputs, weights) in inputs
                        .0
                        .chunks_exact(VEC_SIZE)
                        .zip(weights.chunks_exact(VEC_SIZE))
                    {
                        // SAFETY: input and weights are exactly 256 bits due to chunks_exact.
                        // input and weights are from Align<T> types, which are guaranteed to be aligned.
                        let inputs = _mm256_load_si256(inputs.as_ptr() as *const _);
                        let weights = _mm256_load_si256(weights.as_ptr() as *const _);
                        // u8x32 * i8x32 -> i16x32 horizontal add -> i16x16
                        let partial = _mm256_maddubs_epi16(inputs, weights);
                        // i16x16 * i16x16 -> i32x16 horizontal add -> i32x8
                        // We only want the horizontal add, so we no-op multiply with a vector of all ones.
                        let partial = _mm256_madd_epi16(partial, ones);
                        // i32x8 + i32x8 -> i32x8
                        sum = _mm256_add_epi32(sum, partial);
                    }

                    // Sum i32x8 to i32.
                    // i32x8 lower half -> i32x4
                    let lower = _mm256_castsi256_si128(sum);
                    // i32x8 upper half -> i32x4
                    let upper = _mm256_extracti128_si256::<1>(sum);
                    // i32x4 + i32x4 -> i32x4
                    let sum = _mm_add_epi32(lower, upper);
                    // i32x4 reversed -> i32x4
                    let reversed = _mm_shuffle_epi32(sum, 0b_00_01_10_11);
                    // i32x4 + i32x4 reversed -> i32x2 + ...
                    let sum = _mm_add_epi32(sum, reversed);
                    // i32x2 + ... element 0 -> i32
                    let lower = _mm_cvtsi128_si32(sum);
                    // i32x2 + ... element 1 -> i32
                    let upper = _mm_extract_epi32::<1>(sum);
                    out += lower + upper;
                    return out;
                }
            }
        }
        /*
        #[cfg(target_feature = "neon")]
        {
            use std::arch::aarch64::*;
            const VEC_SIZE: usize = std::mem::size_of::<int8x16_t>() / std::mem::size_of::<u8>();
            // SAFETY: Only enabled on NEON
            if INPUT % VEC_SIZE == 0 {
                unsafe {
                    let weights = &self.weights.0[bucket];
                    let mut sum = vld1q_dup_s32(&0);
                    for (inputs, weights) in inputs
                        .0
                        .chunks_exact(VEC_SIZE)
                        .zip(weights.chunks_exact(VEC_SIZE))
                    {
                        let inputs = vld1q_u8(inputs.as_ptr());
                        let weights = vld1q_s8(weights.as_ptr());

                        let inputs_low = vreinterpretq_s16_u16(vmovl_u8(vget_low_u8(inputs)));
                        let inputs_high = vreinterpretq_s16_u16(vmovl_high_u8(inputs));

                        let weights_low = vmovl_s8(vget_low_s8(weights));
                        let weights_high = vmovl_high_s8(weights);

                        let low_mul = vmulq_s16(inputs_low, weights_low);
                        let high_mul = vmulq_s16(inputs_high, weights_high);
                        let mul_sum = vqaddq_s16(low_mul, high_mul);
                        let low_sum = vmovl_s16(vget_low_s16(mul_sum));
                        let high_sum = vmovl_high_s16(mul_sum);

                        sum = vaddq_s32(sum, vaddq_s32(low_sum, high_sum));
                    }
                    return out + vaddlvq_s32(sum) as i32;
                }
            }
        } */

        let weights = &self.weights.0[bucket];
        for (&input, &weight) in inputs.0.iter().zip(weights) {
            out += weight as i32 * input as i32;
        }
        out
    }
}

pub fn scale_network_output(x: i32) -> i16 {
    (x as i32 * UNITS as i32 / (FT_SCALE as i32 * SCALE as i32)) as i16
}

pub fn sq_clipped_relu<const N: usize>(array: &Align<[i16; N]>, out: &mut [u8]) {
    cfg_if! {
        if #[cfg(target_feature = "neon")]
        {
            for (array, out) in array.0.chunks(256).zip(out.chunks_mut(256)) {
                for (&x, clipped) in array.iter().zip(out.iter_mut()) {
                    let tmp = x.max(MIN).min(MAX) as u16;
                    *clipped = ((tmp * tmp) >> SHIFT) as u8;
                }
            }
        } else {
            for (&x, clipped) in array.0.iter().zip(out.iter_mut()) {
                let tmp = x.max(MIN).min(MAX) as u16;
                *clipped = ((tmp * tmp) >> SHIFT) as u8;
            }
        }
    }
}
