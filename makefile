EXE        = BlackMarlin
EVALFILE   = ./nn/default.bin

rule:
<<<<<<< HEAD
    cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
datagen:
    cargo rustc --release --features data -- -C target-cpu=native --emit link=$(NAME)
=======
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)

datagen:
	cargo rustc --release --features data -- -C target-cpu=native --emit link=$(EXE)
>>>>>>> bddfe8a (Fix makefile)
