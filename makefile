EXE     = BlackMarlin
ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
endif

rule:
	cargo rustc --release --features nnue -- -C target-cpu=native --emit link=$(NAME)