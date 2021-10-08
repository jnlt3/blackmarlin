use std::{env, path::Path};

fn main() {
    let nnue_data = std::fs::read("./nnue.bin").expect("nnue file doesn't exist");
    let (layers, weights) = from_bytes(nnue_data);
    assert_eq!(
        weights.len(),
        2,
        "Blackmarlin only supports NNUEs with a single hidden layer"
    );

    let mut def_nodes = String::new();
    const NODE_NAMES: [&str; 3] = ["INPUT", "MID", "OUTPUT"];
    for (&size, name) in layers.iter().zip(NODE_NAMES) {
        def_nodes += &format!("const {}: usize = {};\n", name, size);
    }
    let mut def_layers = String::new();

    const LAYER_NAMES: [&str; 2] = ["INCREMENTAL", "OUT"];
    for ((weights, name), shape) in weights.iter().zip(LAYER_NAMES).zip(layers.windows(2)) {
        let def_const = format!("const {}: [[i8; {}]; {}] = ", name, shape[1], shape[0]);
        let mut array = "[".to_string();
        for start_range in 0..shape[0] {
            array += "[";
            for &weight in weights[start_range..].iter().step_by(shape[0]).take(shape[1]) {
                array += &format!("{}, ", weight);
            }
            array += "],";
        }
        array += "];";

        def_layers += &def_const;
        def_layers += &array;
        def_layers += "\n";
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("nnue_weights.rs");
    std::fs::write(&dest_path, def_nodes + "\n" + &def_layers).unwrap();
    println!("cargo:rerun-if-changed=./nnue.bin");
}

pub fn from_bytes(bytes: Vec<u8>) -> (Vec<usize>, Vec<Vec<i8>>) {
    let mut layers = vec![];
    for layer_size in bytes.chunks(4).take(3) {
        let layer_size: u32 = unsafe {
            std::mem::transmute([layer_size[0], layer_size[1], layer_size[2], layer_size[3]])
        };
        layers.push(layer_size as usize);
    }
    assert_eq!(
        layers.len(),
        3,
        "Blackmarlin only supports NNUEs with a single hidden layer"
    );

    let mut weights = vec![];
    for layer in layers.windows(2) {
        weights.push(vec![0_i8; layer[0] * layer[1]]);
    }

    let mut bytes_iterator = bytes.iter().skip(layers.len() * std::mem::size_of::<u32>());
    for layer_weights in &mut weights {
        let mut index = 0;
        while let Some(&weight) = bytes_iterator.next() {
            let weight: i8 = unsafe { std::mem::transmute(weight) };
            layer_weights[index] = weight;
            index += 1;
            if index >= layer_weights.len() {
                break;
            }
        }
    }
    assert!(bytes_iterator.next().is_none(), "File not read fully");
    (layers, weights)
}
