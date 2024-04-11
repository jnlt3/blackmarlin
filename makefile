EXE    	= BlackMarlin
EVALFILE = ./nn/default.bin
ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	RUSTFLAGS='-C target-cpu=native' EVALFILE=$(EVALFILE) cargo build --release --bin blackmarlin-uci
	mv ./target/release/blackmarlin-uci $(NAME)
