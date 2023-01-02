BIN=target/riscv32imac-unknown-none-elf/release/usb-prog

all: build check  send

build:
	cargo build --release

check:
	@echo "Size: $$(stat -c%s ${BIN})"

send:
	@echo -n "Uploading... "
	@../tty_send.sh ${BIN}
	@echo "done."
