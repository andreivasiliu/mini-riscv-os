use core::arch::asm;

pub(crate) fn put_char(byte: u8) {
    unsafe {
        asm!(
            "call   usart_put_byte",
            in("a6") byte,
            out("t0") _,
            out("t1") _,
            out("ra") _,
            options(nomem, nostack),
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
            options(nomem, nostack),
        )
    };

    byte
}

pub(crate) fn flash_page_erase(page_number: u8) {
    unsafe {
        asm!(
            "call   flash_page_erase",
            inout("a0") page_number => _,
            out("a1") _,
            out("a2") _,
            out("a3") _,
            options(nostack),
        )
    };
}

pub(crate) fn flash_write(source_page: u8, target_page: u8) {
    unsafe {
        asm!(
            "call   flash_write",
            inout("a0") source_page => _,
            inout("a1") target_page => _,
            out("a2") _,
            out("a3") _,
            out("a4") _,
            out("a5") _,
            out("a6") _,
            options(nostack),
        )
    };
}

pub(crate) fn ecall1(syscall_number: u8, arg1: u32) {
    unsafe {
        asm!(
            "ecall",
            in("a0") syscall_number,
            in("a1") arg1,
            options(nomem, nostack),
        )
    };
}
