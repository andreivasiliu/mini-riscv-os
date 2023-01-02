#![no_std]

use core::arch::asm;


fn ecall1(syscall_number: u8, arg1: u32) {
    unsafe {
        asm!(
            "ecall",
            in("a0") syscall_number,
            in("a1") arg1,
            options(nomem, nostack),
        )
    };
}

pub fn delay(delay: u32) {
    ecall1(1, delay);
}

pub fn set_leds(color_bits: u32) {
    ecall1(2, color_bits);
}

pub fn exec(address: u32) {
    ecall1(3, address);
}

pub fn exit(code: u32) -> ! {
    loop {
        // Note: This never returns.
        // In case it does, just keep calling it; this function must likewise
        // never return.
        ecall1(4, code);
    }
}
