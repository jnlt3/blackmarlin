EXE     = BlackMarlin
ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

rule:
	cargo rustc --release --features nnue -- -C target-cpu=native --emit link=$(NAME)