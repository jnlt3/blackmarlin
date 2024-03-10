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
    let layers = &[768, 1024, 1];

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
