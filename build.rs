use std::{env, path::Path};

fn main() {
    #[cfg(feature = "nnue")]
    parse_bm_net();
}

#[cfg(feature = "nnue")]
fn parse_bm_net() {
    let nnue_data = std::fs::read("./nnue.bin").expect("nnue file doesn't exist");
    let (layers, weights, biases, psqt_weights) = from_bytes_bm(nnue_data);
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
    for (((weights, biases), name), shape) in weights
        .iter()
        .zip(&biases)
        .zip(LAYER_NAMES)
        .zip(layers.windows(2))
    {
        let def_weights = format!("const {}: [[i8; {}]; {}] = ", name, shape[1], shape[0]);
        let mut array = "[".to_string();
        for start_range in 0..shape[0] {
            array += "[";
            for &weight in weights[start_range..]
                .iter()
                .step_by(shape[0])
                .take(shape[1])
            {
                array += &format!("{}, ", weight);
            }
            array += "],";
        }
        array += "];\n";
        def_layers += &def_weights;
        def_layers += &array;

        let def_biases = format!(
            "const {}: [i16; {}] = ",
            name.to_string() + "_BIAS",
            shape[1]
        );
        let mut array = "[".to_string();
        for &weight in biases {
            array += &format!("{}, ", weight);
        }
        array += "];\n";
        def_layers += &def_biases;
        def_layers += &array;
    }

    let def_weights = format!(
        "const PSQT: [[i32; {}]; {}] = ",
        layers[layers.len() - 1],
        layers[0],
    );
    let mut array = "[".to_string();
    for start_range in 0..layers[0] {
        array += "[";
        for &weight in psqt_weights[start_range..]
            .iter()
            .step_by(layers[0])
            .take(layers[layers.len() - 1])
        {
            array += &format!("{}, ", weight);
        }
        array += "],";
    }
    array += "];\n";
    def_layers += &def_weights;
    def_layers += &array;

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("nnue_weights.rs");
    std::fs::write(&dest_path, def_nodes + "\n" + &def_layers).unwrap();
    println!("cargo:rerun-if-changed=./nnue.bin");
}

#[cfg(feature = "nnue")]
pub fn from_bytes_bm(bytes: Vec<u8>) -> (Vec<usize>, Vec<Vec<i8>>, Vec<Vec<i8>>, Vec<i32>) {
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
    let mut biases = vec![];

    for layer in layers.windows(2) {
        weights.push(vec![0_i8; layer[0] * layer[1]]);
        biases.push(vec![0_i8; layer[1]]);
    }

    let mut bytes_iterator = bytes.iter().skip(layers.len() * std::mem::size_of::<u32>());
    for (layer, (layer_weights, bias_weights)) in weights.iter_mut().zip(&mut biases).enumerate() {
        let mut index = 0;
        while let Some(&weight) = bytes_iterator.next() {
            let weight: i8 = unsafe { std::mem::transmute(weight) };
            layer_weights[index] = weight;
            index += 1;
            if index >= layer_weights.len() {
                break;
            }
        }
        if layer != 1 {
            let mut index = 0;
            while let Some(&weight) = bytes_iterator.next() {
                let weight: i8 = unsafe { std::mem::transmute(weight) };
                bias_weights[index] = weight;
                index += 1;
                if index >= bias_weights.len() {
                    break;
                }
            }
        }
    }
    let mut psqt_weights = vec![0_i32; layers[0] * layers[layers.len() - 1]];
    println!("{}", psqt_weights.len());
    let mut index = 0;
    while index < psqt_weights.len() {
        let weight: i32 = unsafe {
            std::mem::transmute([
                *bytes_iterator.next().unwrap(),
                *bytes_iterator.next().unwrap(),
                *bytes_iterator.next().unwrap(),
                *bytes_iterator.next().unwrap(),
            ])
        };
        psqt_weights[index] = weight;
        index += 1;
        if index >= psqt_weights.len() {
            break;
        }
    }
    assert!(bytes_iterator.next().is_none(), "File not read fully");
    (layers, weights, biases, psqt_weights)
}
