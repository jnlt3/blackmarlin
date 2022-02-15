EXE     = BlackMarlin
EVAL_FILE = ./nn/default.bin
ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

rule:
	cp $(EVAL_FILE) nnue.bin
	cargo rustc --release --features nnue -- -C target-cpu=native --emit link=$(NAME)
	rm nnue.bin