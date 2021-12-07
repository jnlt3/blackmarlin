# Blackmarlin

WIP UCI Chess Engine

|![](logo.jpg?raw=true "Black Marlin")|
|:--:|
|*Logo by Alex Brunetti*|


Blackmarlin is a UCI compliant chess engine fully written in Rust.

Make sure to compile the chess engine with `cargo build --release --features 'nnue'` or run it with `cargo run --release --features 'nnue'` in order to get the maximum strength out of the engine.

Blackmarlin doesn't come with a built-in GUI. The recommended way of playing against the engine is to get the latest release or compile it locally and use it along with a Chess GUI that supports the UCI protocol. 

The repository used for NN training is [NNUE Marlin](https://github.com/dsekercioglu/nnue_marlin)
