EXE     = BlackMarlin
rule:
	ifeq ($(OS),Windows_NT)
		EXE := $(EXE).exe
	cargo rustc --release --features nnue -- -C target-cpu=native --emit link=$(EXE)