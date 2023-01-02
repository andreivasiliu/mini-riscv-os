all: v32 push-serial

RUST_LIB=target/riscv32imac-unknown-none-elf/release/libmini_riscv_os.a

v32:
	cargo build --release
	${AS} -march=rv32imac_zicsr -mabi=ilp32 src/start.s -o start.o
	${LD} -flto -Oz -m elf32lriscv -T src/memory.ld start.o ${RUST_LIB} -o start.elf
	${OBJCOPY} -O binary start.elf start.bin

${RUST_LIB}: src/lib.rs
	cargo build --release

run:
	qemu-system-riscv64 -nographic -machine virt -bios none -kernel ./start

push-dfu:
	dfu-util -a 0 -s 0x08000000:leave -D start.bin

push-serial:
	stm32flash -e 30 -g 0x08000000 -b 115200 -w start.bin /dev/ttyUSB0

disas:
	riscv64-unknown-elf-objdump -z --disassemble-all start.elf | less
