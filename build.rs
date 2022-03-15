use std::{env, path::Path};

fn main() {
    parse_policy_net();
    parse_bm_net();
}

fn parse_policy_net() {
    let nn_dir = env::var("POLICYFILE").unwrap();
    let nn_bytes = std::fs::read(nn_dir).expect("nnue file doesn't exist");
    let mut nn_bytes = &nn_bytes[..];

    const HIDDEN: usize = 64;
    let perceptron_weights_0 = dense_from_bytes_i8(&nn_bytes, 768, HIDDEN);
    nn_bytes = &nn_bytes[768 * HIDDEN..];
    let perceptron_bias_0 = bias_from_bytes_i8(&nn_bytes, HIDDEN);
    nn_bytes = &nn_bytes[HIDDEN..];
    let perceptron_weights_1 = dense_rev_from_bytes_i8(&nn_bytes, HIDDEN, 448);
    nn_bytes = &nn_bytes[HIDDEN * 448..];
    let perceptron_bias_1 = bias_from_bytes_i8(&nn_bytes, 448);

    let mut def_layers = String::new();
    def_layers += &format!(
        "pub const P_WEIGHTS_0: [[i8; {}]; 768] = {}\n",
        HIDDEN, perceptron_weights_0,
    );
    def_layers += &format!(
        "pub const P_BIAS_0: [i16; {}] = {}\n",
        HIDDEN, perceptron_bias_0
    );
    def_layers += &format!(
        "pub const P_WEIGHTS_1: [[i8; {}]; 448] = {}\n",
        HIDDEN, perceptron_weights_1,
    );
    def_layers += &format!("pub const P_BIAS_1: [i16; 448] = {}\n", perceptron_bias_1);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("policy_weights.rs");
    std::fs::write(&dest_path, &def_layers).unwrap();
    println!("cargo:rerun-if-changed=./nnue.bin");
}

fn parse_bm_net() {
    let nn_dir = env::var("EVALFILE").unwrap();
    let nn_bytes = std::fs::read(nn_dir).expect("nnue file doesn't exist");

    let layers = parse_arch(&nn_bytes);
    let mut nn_bytes = &nn_bytes[12..];

    let mut def_nodes = String::new();
    const NODE_NAMES: [&str; 3] = ["INPUT", "MID", "OUTPUT"];
    for (&size, name) in layers.iter().zip(NODE_NAMES) {
        def_nodes += &format!("const {}: usize = {};\n", name, size);
    }
    let mut def_layers = String::new();

    let incremental = dense_from_bytes_i8(&nn_bytes, layers[0], layers[1]);
    nn_bytes = &nn_bytes[layers[0] * layers[1]..];

    let incremental_bias = bias_from_bytes_i8(&nn_bytes, layers[1]);
    nn_bytes = &nn_bytes[layers[1]..];

    let out = dense_from_bytes_i8(&nn_bytes, layers[1], layers[2]);
    nn_bytes = &nn_bytes[layers[1] * layers[2]..];

    let out_bias = bias_from_bytes_i8(&nn_bytes, layers[2]);
    nn_bytes = &nn_bytes[layers[2]..];

    let res = dense_from_bytes_i32(&nn_bytes, layers[0], layers[2]);
    nn_bytes = &nn_bytes[layers[0] * layers[2] * 4..];

    def_layers += &format!(
        "pub const INCREMENTAL: [[i8; {}]; {}] = {}\n",
        layers[1], layers[0], incremental
    );
    def_layers += &format!(
        "pub const INCREMENTAL_BIAS: [i16; {}] = {}\n",
        layers[1], incremental_bias
    );
    def_layers += &format!(
        "pub const OUT: [[i8; {}]; {}] = {}\n",
        layers[2], layers[1], out
    );
    def_layers += &format!("pub const OUT_BIAS: [i32; {}] = {}\n", layers[2], out_bias);
    def_layers += &format!(
        "pub const PSQT: [[i32; {}]; {}] = {}\n",
        layers[2], layers[0], res
    );

    assert!(nn_bytes.is_empty(), "{}", nn_bytes.len());

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("nnue_weights.rs");
    std::fs::write(&dest_path, def_nodes + "\n" + &def_layers).unwrap();
    println!("cargo:rerun-if-changed=./nnue.bin");
}

pub fn parse_arch(bytes: &[u8]) -> [usize; 3] {
    let mut layers = [0; 3];
    for (bytes, layer) in bytes.chunks(4).take(3).zip(&mut layers) {
        *layer = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    }
    layers
}

pub fn dense_from_bytes_i8(bytes: &[u8], input: usize, output: usize) -> String {
    let mut weights = vec![];
    for &byte in bytes.iter().take(input * output) {
        weights.push(i8::from_le_bytes([byte]))
    }
    let mut array = "[".to_string();
    for weights in weights.chunks(output) {
        array += "[";
        for &weight in weights {
            array += &format!("{}, ", weight);
        }
        array += "],";
    }
    array += "];";
    array
}

pub fn dense_rev_from_bytes_i8(bytes: &[u8], input: usize, output: usize) -> String {
    let mut weights = vec![];
    for &byte in bytes.iter().take(input * output) {
        weights.push(i8::from_le_bytes([byte]))
    }
    let mut array = "[".to_string();
    for index in 0..output {
        array += "[";
        for &weight in weights[index..].iter().step_by(output).take(input) {
            array += &format!("{}, ", weight);
        }
        array += "],";
    }
    array += "];";
    array
}

pub fn dense_from_bytes_i32(bytes: &[u8], input: usize, output: usize) -> String {
    let mut weights = vec![];
    for bytes in bytes.chunks(4).take(input * output) {
        weights.push(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    let mut array = "[".to_string();
    for weights in weights.chunks(output) {
        array += "[";
        for &weight in weights {
            array += &format!("{}, ", weight);
        }
        array += "],";
    }
    array += "];";
    array
}

pub fn bias_from_bytes_i8(bytes: &[u8], len: usize) -> String {
    let mut weights = vec![];
    for &byte in bytes.iter().take(len) {
        weights.push(i8::from_le_bytes([byte]))
    }
    let mut array = "[".to_string();
    for weight in weights {
        array += &format!("{}, ", weight);
    }
    array += "];";
    array
}
