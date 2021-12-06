EXE     = BlackMarlin
rule:
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)