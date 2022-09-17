use core::arch::asm;

pub(crate) trait Printable {
    fn print(&self);
}

impl Printable for &[u8] {
    fn print(&self) {
        put_string(self);
    }
}

impl Printable for &str {
    fn print(&self) {
        put_string(self.as_bytes());
    }
}

impl Printable for i32 {
    fn print(&self) {
        put_decimal(*self);
    }
}

impl Printable for u8 {
    fn print(&self) {
        put_char(*self);
    }
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

fn put_decimal(mut d: i32) {
    if d < 0 {
        put_char(b'-');
        d *= -1;
    }

    let first_digit = d % 10;
    d /= 10;

    let mut reversed_d = 0;
    let mut extra_digits = 0;

    while d != 0 {
        extra_digits += 1;
        reversed_d = reversed_d * 10 + d % 10;
        d /= 10;
    }

    while extra_digits != 0 {
        extra_digits -= 1;
        put_char((reversed_d % 10) as u8 + b'0');
        reversed_d /= 10;
    }

    put_char(first_digit as u8 + b'0');
}

pub(crate) fn put_printable(p: impl Printable) {
    p.print();
}

#[macro_export]
macro_rules! put {
    ($expression:expr) => {
        crate::print::put_printable($expression);
        crate::print::put_printable("\r\n");
    };

    ($expression:expr, $($rest:expr),+) => {
        crate::print::put_printable($expression);
        crate::print::put_printable(" ");
        put!( $($rest),+);
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
