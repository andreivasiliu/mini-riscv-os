use core::{panic::PanicInfo, arch::asm};

use crate::put;

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    put!("Panicked!");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
