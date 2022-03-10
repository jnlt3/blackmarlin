EXE    	= BlackMarlin
EVALFILE = ./nn/default.bin
POLICYFILE = ./nn/policy.bin
ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

rule:
	POLICYFILE=$(POLICYFILE) EVALFILE=$(EVALFILE) cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

debug:
	POLICYFILE=$(POLICYFILE) EVALFILE=$(EVALFILE) cargo rustc -- -C target-cpu=native --emit link=$(NAME)