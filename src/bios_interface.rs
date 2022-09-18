use core::arch::asm;

pub(crate) fn put_char(byte: u8) {
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

pub(crate) fn get_char() -> u8 {
    let byte;

    unsafe {
        asm!(
            "call   usart_get_byte",
            out("a6") byte,
            out("t0") _,
            out("t1") _,
            out("ra") _,
        )
    };

    byte
}

pub(crate) fn ecall() {
    unsafe {
        asm!(
            "ecall",
            // out("a0") _,
            // out("a1") _,
            // out("ra") _,
        )
    };
}
