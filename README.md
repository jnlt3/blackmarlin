# Blackmarlin

WIP XBoard Chess Engine

Blackmarlin is a chess engine fully written in Rust.

Make sure to compile the chess engine with `cargo build --release --features "standard jem"` or run it with `cargo run --release --features "standard jem"` in order to get the maximum strength out of the engine. If build fails, remove the jem option and try again.

In order to access experimental features that aren't yet used, use `experimental` instead of `standard` when building.

Blackmarlin doesn't come with a built-in GUI. The recommended way of playing against the engine is to compile it locally and use it along with a Chess GUI that supports the Xboard protocol. 
