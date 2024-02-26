EXE        = BlackMarlin
EVALFILE   = ./nn/default.bin

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

datagen:
	cargo rustc --release --features data -- -C target-cpu=native --emit link=$(NAME)