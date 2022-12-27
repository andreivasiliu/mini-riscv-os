use crate::bios_interface::ecall1;


pub(crate) fn set_leds(colors: &[u8]) {
    let mut color_bits = 0b111000;

    for color in colors {
        match color {
            b'r' | b'1' => {
                color_bits &= 0b110111;
                color_bits |= 0b000001;
            }
            b'g' | b'2' => {
                color_bits &= 0b101111;
                color_bits |= 0b000010;
            }
            b'b' | b'3' => {
                color_bits &= 0b011111;
                color_bits |= 0b000100;
            }
            _ => (),
        }
    }

    ecall1(2, color_bits);
}

pub(crate) fn delay(delay: u32) {
    ecall1(1, delay);
}

pub(crate) fn exec(address: u32) {
    ecall1(3, address);
}

pub(crate) fn exit(code: u32) {
    ecall1(4, code);
}
