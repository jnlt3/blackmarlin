use std::{env, path::Path};

fn main() {
    parse_bm_net();
}

fn parse_bm_net() {
    let nn_dir = env::var("EVALFILE").unwrap_or_else(|_| "./nn/default.bin".to_string());
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let eval_path = Path::new(&out_dir).join("eval.bin");
    let nn_bytes = std::fs::read(nn_dir).expect("nnue file doesn't exist");
    let layers = parse_arch(&nn_bytes);

    let arch_path = Path::new(&out_dir).join("arch.rs");
    let mut def_nodes = String::new();
    const LAYER_SIZES: [&str; 4] = ["INPUT", "HIDDEN_0", "HIDDEN_1", "OUTPUT"];
    for (&size, name) in layers.iter().zip(LAYER_SIZES) {
        def_nodes += &format!("const {}: usize = {};\n", name, size);
    }

    std::fs::write(&eval_path, nn_bytes).unwrap();
    std::fs::write(&arch_path, def_nodes).unwrap();
}

pub fn parse_arch(bytes: &[u8]) -> [usize; 4] {
    let mut layers = [0; 4];
    for (bytes, layer) in bytes.chunks(4).take(4).zip(&mut layers) {
        *layer = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    }
    layers
}
