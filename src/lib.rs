#![no_std]
#![no_main]

use file_system::FileSystem;
use xmodem::receive_file;

use crate::{
    bios_interface::{ecall1, get_char},
    edit_line::{EditLine, EditLineEvent},
};

mod bios_interface;
mod edit_line;
mod file_system;
mod panic;
mod print;
mod xmodem;
mod sys_call;

#[no_mangle]
fn os_main() {
    put!("Hi from Rust!");

    put!("Number from Rust:", 1234);

    put_prompt();

    let mut edit_line = EditLine::new();
    let mut file_system = FileSystem::load_from_flash();

    loop {
        let input_character = get_char();

        match edit_line.input_character(input_character) {
            None => (),
            Some(EditLineEvent::EscapeCode(escape_code)) => {
                process_escape_code(escape_code);
                put_prompt();
            }
            Some(EditLineEvent::Command(command)) => {
                process_command(command, &mut file_system);
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

#[inline(never)]
fn process_command(command: &[u8], file_system: &mut &mut FileSystem) {
    if command.is_empty() {
        return;
    }

    let (command, args) = get_word(command);

    match command {
        b"help" => {
            put!("No help available at this time.");
        }
        b"ecall" => {
            put!("Calling system interrupt...");
            let (arg1, _) = get_word(args);
            let arg1 = string_to_number(arg1);
            ecall1(1, arg1);
            put!("Back to Rust now.");
        }
        b"leds" => {
            let (arg1, _) = get_word(args);
            sys_call::set_leds(arg1);
        }
        b"delay" => {
            let (arg1, _) = get_word(args);
            let milliseconds = string_to_number(arg1);
            sys_call::delay(milliseconds);
        }
        b"fs" => match args {
            b"stats" => {
                file_system.print_stats();
            }
            b"save" => {
                file_system.save_file_system();
            }
            b"load" => {
                *file_system = FileSystem::load_from_flash();
            }
            b"reset" => {
                *file_system = FileSystem::new_from_scratch();
            }
            _ => {
                put!("Unknown argument:", args);
                put!("Subcommands: stats, save, load, reset");
            }
        },
        b"write" | b"create" => {
            let (file_name, content) = get_word(args);

            if let None = file_system.create_file(file_name, content) {
                put!("No file created.");
            }
        }
        b"ls" => {
            let (file_name_arg, _) = get_word(args);

            for file in file_system.list_files() {
                let file_name = file_system.file_name(file);
                if file_name_arg.is_empty() || file_name == file_name_arg {
                    put!(file_name);
                }
            }
        }
        b"cat" | b"read" => {
            let (file_name, _) = get_word(args);

            if let Some(file) = file_system.file(file_name) {
                put!(file_system.read_file(file));
            } else {
                put!("File not found:", file_name);
            }
        }
        b"rm" => {
            let (file_name, _) = get_word(args);

            let file = file_system
                .list_files()
                .find(|&file| file_system.file_name(file) == file_name);

            if let Some(file) = file {
                file_system.remove_file(file);
            } else {
                put!("File not found:", file_name);
            }
        }
        b"rx" => {
            receive_file(file_system);
        }
        b"paste" => {
            let (file_name, rest) = get_word(args);
            let (file_size, _) = get_word(rest);
            let file_size = string_to_number(file_size) as usize;

            file_system.paste_file(file_name, file_size);
        }
        c => put!("Unknown command:", c),
    }
}

fn get_word(s: &[u8]) -> (&[u8], &[u8]) {
    if s.is_empty() {
        return (s, b"");
    }

    for (index, &byte) in s.iter().enumerate() {
        if byte.is_ascii_whitespace() {
            let (word, rest) = s.split_at(index % s.len());

            if rest.is_empty() {
                return (word, b"");
            }

            return (word, &rest[1..]);
        }
    }

    return (s, b"");
}

fn string_to_number(s: &[u8]) -> u32 {
    let mut number = 0;

    for &byte in s {
        if byte >= b'0' && byte <= b'9' {
            let digit = byte - b'0';

            number *= 10;
            number += digit as u32;
        }
    }

    number
}
