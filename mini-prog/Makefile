all: build

build:
	riscv64-unknown-elf-gcc -march=rv32imac -mabi=ilp32 -O2 -nostdlib -nostdinc prog.c

send:
	../mini-riscv-os/tty_send.sh a.out

disas:
	riscv64-unknown-elf-objdump -d a.out
