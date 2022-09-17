// #define UART        0x10000000
// #define UART_THR    (uint8_t*)(UART+0x00) // THR:transmitter holding register
// #define UART_LSR    (uint8_t*)(UART+0x05) // LSR:line status register
// #define UART_LSR_EMPTY_MASK 0x40          // LSR Bit 6: Transmitter empty; both the THR and LSR are empty

#![no_std]
#![no_main]

extern "C" {}

use core::{arch::asm, panic::PanicInfo};

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}

fn put_char(byte: u8) {
    unsafe {
        asm!(
            "call   usart_put_byte",
            in("a6") byte,
            out("t0") _,
            out("t1") _,
            out("ra") _,
        )
    };
}

fn put_string(s: &[u8]) {
    for &byte in s {
        put_char(byte);
    }
}

#[no_mangle]
fn os_main() {
    put_string("Hi from Rust!\r\n".as_bytes());
}
