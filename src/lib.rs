#![no_std]
#![no_main]

use crate::bios_interface::{get_char, put_char, ecall};

mod panic;
mod print;
mod bios_interface;

#[no_mangle]
fn os_main() {
    put!("Hi from Rust!");

    put!("Number from Rust:", 1234);

    let mut command = [0; 256];
    let mut cursor = 0;
    let mut inside_escape_code = false;
    let mut escape_code = [0; 16];
    let mut escape_cursor = 0;

    put_prompt();

    loop {
        let input_character = get_char();

        match input_character {
            c if inside_escape_code => {
                unsafe {
                    *escape_code.get_unchecked_mut(escape_cursor) = c;
                }
                escape_cursor += 1;

                if c.is_ascii_alphabetic() || escape_cursor > 15 {
                    inside_escape_code = false;
                    let escape_code = unsafe {
                        escape_code.get_unchecked(..escape_cursor)
                    };
                    process_escape_code(escape_code);
                    escape_cursor = 0;
                    put_prompt();
                }
            }
            c if is_printable(c) => {
                unsafe { *command.get_unchecked_mut(cursor) = c };
                cursor += 1;
                put_char(c);
            }
            b'\r' => {
                put!();
                let command = unsafe { command.get_unchecked(..cursor) };
                cursor = 0;
                process_command(command);
                put_prompt();
            }
            8 | 127 => {
                // Backspace
                if cursor > 0 {
                    cursor -= 1;
                    putn!(8u8, b' ', 8u8);
                }
            }
            27 => {
                inside_escape_code = true;
            }
            c => {
                put!();
                cursor = 0;
                put!("Unrecognized ascii code:", c as u32 as i32);
                put_prompt();
            },
        }
    }
}

fn process_escape_code(escape_code: &[u8]) {
    put!();
    match escape_code {
        b"[D" => put!("Escape code: Left key"),
        b"[C" => put!("Escape code: Right key"),
        b"[A" => put!("Escape code: Up key"),
        b"[B" => put!("Escape code: Down key"),
        _ => {
            putn!("Unrecognized escape code: Esc");
    
            for &byte in escape_code {
                if byte.is_ascii_graphic() {
                    putn!(" '", byte, "'");
                } else {
                    putn!(" ", byte as u32 as i32);
                }
            }

            put!();
        }
    }
}

fn is_printable(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_digit() || c == b' ' || c.is_ascii_punctuation()
}

fn put_prompt() {
    putn!("\x1b[1;34m>\x1b[0m ");
}

fn process_command(command: &[u8]) {
    if command.is_empty() {
        return;
    }

    match command {
        b"help" => {
            put!("No help available at this time.");
        }
        b"ecall" => {
            put!("Calling system interrupt...");
            ecall();
            put!("Back to Rust now.");
        }
        c => put!("Unknown command:", c),
    }
}
