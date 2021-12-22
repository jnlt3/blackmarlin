EXE     = BlackMarlin

ifeq ($(OS),Windows_NT)
	EXE := $(EXE).exe

rule:
	cargo rustc --release --features nnue -- -C target-cpu=native --emit link=$(EXE)