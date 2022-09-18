#![no_std]
#![no_main]

use crate::{bios_interface::{get_char, ecall}, edit_line::{EditLine, EditLineEvent}};

mod panic;
mod print;
mod bios_interface;
mod edit_line;

#[no_mangle]
fn os_main() {
    put!("Hi from Rust!");

    put!("Number from Rust:", 1234);

    put_prompt();

    let mut edit_line = EditLine::new();

    loop {
        let input_character = get_char();

        match edit_line.input_character(input_character) {
            None => (),
            Some(EditLineEvent::EscapeCode(escape_code)) => {
                process_escape_code(escape_code);
                put_prompt();
            }
            Some(EditLineEvent::Command(command)) => {
                process_command(command);
                put_prompt();
            }
            Some(EditLineEvent::UnrecognizedCode(c)) => {
                put!();
                put!("Unrecognized ascii code:", c as u32 as i32);
                put_prompt();
            }
        }
    }
}

fn put_prompt() {
    putn!("\x1b[1;34m>\x1b[0m ");
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
