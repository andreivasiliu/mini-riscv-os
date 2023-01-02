#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    syscall::exit(1);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    syscall::set_leds(0b010);

    syscall::exit(0);
}
