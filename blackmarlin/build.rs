use std::fmt::Write;
use std::{env, path::Path};

fn main() {
    parse_bm_net();
}

fn parse_bm_net() {
    let nn_dir = env::var("EVALFILE").unwrap_or_else(|_| "./nn/default.bin".to_string());
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let eval_path = Path::new(&out_dir).join("eval.bin");
    let nn_bytes = std::fs::read(&nn_dir).expect("nnue file doesn't exist");
    let layers = parse_arch(&nn_bytes);

    let arch_path = Path::new(&out_dir).join("arch.rs");
    let mut def_nodes = String::new();
    const LAYER_SIZES: [&str; 3] = ["INPUT", "MID", "OUTPUT"];
    for (&size, name) in layers.iter().zip(LAYER_SIZES) {
        writeln!(&mut def_nodes, "const {}: usize = {};", name, size).unwrap();
    }

    std::fs::write(&eval_path, nn_bytes).unwrap();
    std::fs::write(&arch_path, def_nodes).unwrap();

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={nn_dir}");
}

pub fn parse_arch(bytes: &[u8]) -> [usize; 3] {
    let mut layers = [0; 3];
    for (bytes, layer) in bytes.chunks(4).take(3).zip(&mut layers) {
        let bytes = bytes.try_into().unwrap();
        *layer = u32::from_le_bytes(bytes) as usize;
    }
    layers
}
