#![no_std]
#![no_main]

use crate::print::get_char;

mod panic;
mod print;

#[no_mangle]
fn os_main() {
    put!("Hi from Rust!");

    put!("Number from Rust:", 1234);

    loop {
        let c = get_char();
        put!("You typed:", c);
    }
}
