EXE    	= BlackMarlin
EVALFILE = ./nn/default.bin
ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	EVALFILE=$(EVALFILE) cargo build --release --bin blackmarlin-uci && cp ./target/release/blackmarlin-uci $(NAME)