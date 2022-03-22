pub fn dense_from_bytes_i8<const INPUT: usize, const OUTPUT: usize>(
    bytes: &[u8],
) -> [[i8; OUTPUT]; INPUT] {
    let mut weights = vec![];
    for &byte in bytes.iter().take(INPUT * OUTPUT) {
        weights.push(i8::from_le_bytes([byte]))
    }
    let mut dense = [[0; OUTPUT]; INPUT];
    for (i, weights) in weights.chunks(OUTPUT).enumerate() {
        for (j, &weight) in weights.into_iter().enumerate() {
            dense[i][j] = weight;
        }
    }
    dense
}

pub fn bias_from_bytes_i8<const LEN: usize, T: From<i8> + Copy + Default>(
    bytes: &[u8],
) -> [T; LEN] {
    let mut weights = [T::default(); LEN];
    for (&byte, weight) in bytes.iter().zip(&mut weights).take(LEN) {
        *weight = T::from(i8::from_le_bytes([byte]));
    }
    weights
}
