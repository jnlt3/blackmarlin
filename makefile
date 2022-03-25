EXE    	= BlackMarlin
EVALFILE = ./nn/default.bin
ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

rule:
	EVALFILE=$(EVALFILE) cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
datagen:
	EVALFILE=$(EVALFILE) cargo rustc --release --features data -- -C target-cpu=native --emit link=$(NAME)