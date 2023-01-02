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
    // Note: This never returns.
    ecall1(4, code);
    // In case it does, just loop forever in power-saving mode.
    loop {
        // Wait for interrupt.
        unsafe {
            asm!("wfi");
        }
    }
}

pub fn put_byte(byte: u8) {
    ecall1(5, byte.into());
}
