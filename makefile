EXE    	= BlackMarlin
EVALFILE = ./blackmarlin/nn/default.bin
ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	EVALFILE=../$(EVALFILE) cargo rustc --package blackmarlin-uci --release -- -C target-cpu=native -C lto=thin --emit link=$(NAME)