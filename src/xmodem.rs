use core::arch::asm;

use crate::{bios_interface::get_char, file_system::FileSystem, put, putn};

const SOH: u8 = 0x01; // Start of Header
const EOT: u8 = 0x04; // End of Transmission
const ACK: u8 = 0x06; // Acknowledge
const NAK: u8 = 0x15; // Not Acknowledge
const CAN: u8 = 0x18; // Cancel

fn delay(time: u32) {
    for _index in 0..100000 * time {
        unsafe {
            asm!("", options(nomem, nostack));
        }
    }
}

pub(crate) fn receive_file(_file_system: &FileSystem) {
    let mut buffer = [0; 1024];
    let mut blocks: i32 = 0;
    let mut check_sum_ok: i32 = 0;

    put!("Preparing to receive...");

    delay(100);

    putn!(NAK);

    loop {
        let mut check_sum: u8 = 0;

        let c = get_char();

        if c == EOT {
            putn!(ACK);
            delay(1);
            put!("Receive successful.");
            put!("Received blocks:", blocks);
            put!("Successful checksums:", check_sum_ok);
            break;
        }

        if c != SOH {
            putn!(CAN);
            put!("Error: Packet did not start with a SOH.");
            put!("Instead it was:", c as u32 as i32);
            break;
        }

        let _block_id = get_char();
        let _block_id_reversed = get_char();

        for index in 0..128 {
            let byte = get_char();
            buffer[index] = byte;

            check_sum = check_sum.wrapping_add(byte);
        }

        let expected_check_sum = get_char();

        blocks += 1;
        if check_sum == expected_check_sum {
            check_sum_ok += 1;
        }

        putn!(ACK);
    }
}
