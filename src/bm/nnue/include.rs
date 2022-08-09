use super::layers::Aligned;

pub fn sparse_from_bytes_i16<
    T: From<i16> + Copy + Default,
    const INPUT: usize,
    const OUTPUT: usize,
>(
    bytes: &[u8],
) -> Box<Aligned<[[T; OUTPUT]; INPUT]>> {
    let mut weights = vec![];
    for bytes in bytes.chunks(2).take(INPUT * OUTPUT) {
        weights.push(i16::from_le_bytes([bytes[0], bytes[1]]))
    }
    let mut dense = Box::new(Aligned([[T::default(); OUTPUT]; INPUT]));
    for (i, weights) in weights.chunks(OUTPUT).enumerate() {
        for (j, &weight) in weights.iter().enumerate() {
            dense.0[i][j] = T::from(weight);
        }
    }
    dense
}

pub fn bias_from_bytes_i16<T: From<i16> + Copy + Default, const LEN: usize>(
    bytes: &[u8],
) -> Aligned<[T; LEN]> {
    let mut weights = Aligned([T::default(); LEN]);
    for (bytes, weight) in bytes.chunks(2).zip(&mut weights.0).take(LEN) {
        *weight = T::from(i16::from_le_bytes([bytes[0], bytes[1]]));
    }
    weights
}

pub fn dense_from_bytes_i8<
    T: From<i8> + Copy + Default,
    const INPUT: usize,
    const OUTPUT: usize,
>(
    bytes: &[u8],
) -> Box<Aligned<[[T; INPUT]; OUTPUT]>> {
    let mut weights = vec![];
    for &byte in bytes.iter().take(INPUT * OUTPUT) {
        weights.push(i8::from_le_bytes([byte]))
    }
    let mut dense = Box::new(Aligned([[T::default(); INPUT]; OUTPUT]));
    for (i, weights) in weights.chunks(INPUT).enumerate() {
        for (j, &weight) in weights.iter().enumerate() {
            dense.0[i][j] = T::from(weight);
        }
    }
    dense
}
