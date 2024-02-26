EXE        = BlackMarlin
EVALFILE   = ./nn/default.bin

rule:
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)

datagen:
	cargo rustc --release --features data -- -C target-cpu=native --emit link=$(EXE)