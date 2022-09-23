all: v32 push

RUST_LIB=target/riscv32imac-unknown-none-elf/release/libmini_riscv_os.a

v32:
	cargo build --release
	riscv64-unknown-elf-as -march=rv32im -mabi=ilp32 src/start.s -o start.o
	riscv64-unknown-elf-ld -flto -Oz -m elf32lriscv -T src/memory.ld start.o ${RUST_LIB} -o start.elf
	riscv64-unknown-elf-objcopy -O binary start.elf start.bin

${RUST_LIB}: src/lib.rs
	cargo build --release

v64:
	riscv64-unknown-elf-as -march=rv64imac src/start.old.s -o start.o
	riscv64-unknown-elf-ld -T src/memory.ld start.o -o start
	riscv64-unknown-elf-objcopy -O binary start start.bin

run:
	qemu-system-riscv64 -nographic -machine virt -bios none -kernel ./start

push:
	dfu-util -a 0 -s 0x08000000:leave -D start.bin

disas:
	riscv64-unknown-elf-objdump -z --disassemble-all start.elf | less
