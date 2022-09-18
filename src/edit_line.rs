use crate::{bios_interface::put_char, put, putn};

pub(crate) struct EditLine {
    command: [u8; 256],
    cursor: usize,
    inside_escape_code: bool,
    escape_code: [u8; 16],
    escape_cursor: usize,
}

impl EditLine {
    pub(crate) fn new() -> Self {
        Self {
            command: [0; 256],
            cursor: 0,
            inside_escape_code: false,
            escape_code: [0; 16],
            escape_cursor: 0,
        }
    }

    pub(crate) fn input_character(&mut self, c: u8) -> Option<EditLineEvent> {
        process_character(self, c)
    }
}

pub(crate) enum EditLineEvent<'a> {
    Command(&'a [u8]),
    EscapeCode(&'a [u8]),
    UnrecognizedCode(u8),
}

pub(crate) fn process_character(edit_line: &mut EditLine, input_character: u8) -> Option<EditLineEvent> {
    match input_character {
        c if edit_line.inside_escape_code => {
            unsafe {
                *edit_line.escape_code.get_unchecked_mut(edit_line.escape_cursor) = c;
            }
            edit_line.escape_cursor += 1;

            if c.is_ascii_alphabetic() || edit_line.escape_cursor > 15 {
                edit_line.inside_escape_code = false;
                let escape_code = unsafe {
                    edit_line.escape_code.get_unchecked(..edit_line.escape_cursor)
                };
                edit_line.escape_cursor = 0;
                Some(EditLineEvent::EscapeCode(escape_code))
            } else {
                None
            }
        }
        c if is_printable(c) => {
            unsafe { *edit_line.command.get_unchecked_mut(edit_line.cursor) = c };
            edit_line.cursor += 1;
            put_char(c);
            None
        }
        b'\r' => {
            put!();
            let command = unsafe { edit_line.command.get_unchecked(..edit_line.cursor) };
            edit_line.cursor = 0;
            Some(EditLineEvent::Command(command))
        }
        8 | 127 => {
            // Backspace
            if edit_line.cursor > 0 {
                edit_line.cursor -= 1;
                putn!(8u8, b' ', 8u8);
            }
            None
        }
        27 => {
            edit_line.inside_escape_code = true;
            None
        }
        c => {
            edit_line.cursor = 0;
            Some(EditLineEvent::UnrecognizedCode(c))
        },
    }
}

fn is_printable(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_digit() || c == b' ' || c.is_ascii_punctuation()
}
