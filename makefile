EXE     = BlackMarlin
rule:
	cargo rustc --release --features 'nnue' -- -C target-cpu=native --emit link=$(EXE)