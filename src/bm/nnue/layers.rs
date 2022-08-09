use std::sync::Arc;

const UNITS: i16 = 400_i16;
const FT_SCALE: i16 = 255;
const SCALE: i16 = 64;
const MIN: i16 = 0;
const MAX: i16 = FT_SCALE;
const SHIFT: i16 = 8;

#[repr(align(64))]
#[derive(Debug, Copy, Clone)]
pub struct Aligned<T>(pub T);

#[derive(Debug, Clone)]
pub struct Incremental<const INPUT: usize, const OUTPUT: usize> {
    weights: Arc<Aligned<[[i16; OUTPUT]; INPUT]>>,
    out: Aligned<[i16; OUTPUT]>,
}

impl<const INPUT: usize, const OUTPUT: usize> Incremental<INPUT, OUTPUT> {
    pub fn new(
        weights: Arc<Aligned<[[i16; OUTPUT]; INPUT]>>,
        bias: Aligned<[i16; OUTPUT]>,
    ) -> Self {
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
    weights: Arc<Aligned<[[i8; INPUT]; OUTPUT]>>,
    bias: Aligned<[i32; OUTPUT]>,
}

impl<const INPUT: usize, const OUTPUT: usize> Dense<INPUT, OUTPUT> {
    pub fn new(weights: Arc<Aligned<[[i8; INPUT]; OUTPUT]>>, bias: Aligned<[i32; OUTPUT]>) -> Self {
        Self { weights, bias }
    }

    #[cfg(not(target_feature = "avx2"))]
    pub fn ff(&self, inputs: &[u8; INPUT]) -> [i16; OUTPUT] {
        let mut out = self.bias.0;
        for (out, weights) in out.iter_mut().zip(&self.weights.0) {
            for (&input, &weight) in inputs.iter().zip(weights.iter()) {
                *out += (weight as i16 * input as i16) >> INTERMEDIATE_SHIFT;
            }
        }
        out
    }

    #[cfg(target_feature = "sse")]
    unsafe fn sum_i16(x: std::arch::x86_64::__m128i) -> i32 {
        use std::arch::x86_64::_mm_store_si128;
        const CHUNKS: usize = 128 / 16;
        let mut vector = [0_i16; CHUNKS];
        _mm_store_si128(vector.as_mut_ptr() as *mut _, x);
        vector.into_iter().map(|x| x as i32).sum()
    }

    #[cfg(target_feature = "avx2")]
    pub fn ff(&self, inputs: &[u8; INPUT]) -> [i32; OUTPUT] {
        use std::arch::x86_64::{
            _mm256_add_epi16, _mm256_castsi256_si128, _mm256_extracti128_si256, _mm256_load_si256,
            _mm256_maddubs_epi16, _mm256_setzero_si256, _mm_add_epi16,
        };
        const CHUNKS: usize = 256 / 8;
        let mut out = self.bias.0;

        for (out, weights) in out.iter_mut().zip(&self.weights.0) {
            unsafe {
                let mut sums = _mm256_setzero_si256();
                for (inputs, weights) in inputs.chunks(CHUNKS).zip(weights.chunks(CHUNKS)) {
                    let inputs = _mm256_load_si256(inputs.as_ptr() as *const _);
                    let weights = _mm256_load_si256(weights.as_ptr() as *const _);
                    let out = _mm256_maddubs_epi16(inputs, weights);
                    sums = _mm256_add_epi16(sums, out);
                }
                let low = _mm256_castsi256_si128(sums);
                let high = _mm256_extracti128_si256::<1>(sums);
                let sum = _mm_add_epi16(low, high);
                *out += Self::sum_i16(sum);
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
