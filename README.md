# Blackmarlin

WIP XBoard Chess Engine

Blackmarlin is a chess engine fully written in Rust.

Make sure to compile the chess engine with `cargo build --release --features "jem"` or run it with `cargo run --release --features "jem"` in order to get the maximum strength out of the engine. If build fails, remove the jem option and try again.

Blackmarlin doesn't come with a built-in GUI. The recommended way of playing against the engine is to compile it locally and use it along with a Chess GUI that supports the Xboard protocol. 

Blackmarlin also includes an optional Neural Network as a replacement for hand crafted evaluation. In order to enable this feature, add "nnue" to the feature flags: `cargo build --release --features "nnue jem"`

Using the neural network results in stronger play overall.