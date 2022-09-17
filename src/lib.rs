#![no_std]
#![no_main]

mod panic;
mod print;

#[no_mangle]
fn os_main() {
    put!("Hi from Rust!");

    put!("Number from Rust:", 1234);
}
