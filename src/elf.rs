use crate::put;


pub(crate) fn read_elf(contents: &[u8]) -> Result<usize, &'static str> {
    let elf_header = contents.get(0..54)
        .ok_or("ELF header too short")?;

    let magic_header = &elf_header[0..4];

    if magic_header != b"\x7FELF" {
        return Err("Cannot run program: Magic ELF header not found.");
    }

    let bit_format = elf_header[4];

    if bit_format != 1 {
        return Err("Cannot run program: Not 32-bit.");
    }

    let entry_point = read_u32(&elf_header[0x18..0x18+4]) as usize;

    let program_header_start = read_u32(&elf_header[0x1C..0x1C+4]) as usize;
    let program_header_size = read_u16(&elf_header[0x2A..0x2A+2]) as usize;
    let program_header_count = read_u16(&elf_header[0x2C..0x2C+2]) as usize;

    let program_headers_end = program_header_start + program_header_size * program_header_count;

    let program_headers = contents.get(program_header_start..program_headers_end)
        .ok_or("ELF program headers section too short")?;

    for index in 0..program_header_count {
        let header_start = index * program_header_size;
        let header_end = (index + 1) * program_header_size;

        let program_header = program_headers.get(header_start..header_end)
            .ok_or("ELF program header outside range")?;

        if let Some(address) = read_program_header(program_header, entry_point)? {
            return Ok(address);
        }
    }

    Err("No program headers found")
}

fn read_program_header(program_header: &[u8], entry_point: usize) -> Result<Option<usize>, &'static str> {
    if program_header.len() < 0x14 {
        return Err("Program header too short");
    }

    let file_offset = read_u32(&program_header[0x04..0x04+4]) as usize;
    let virtual_offset = read_u32(&program_header[0x08..0x08+4]) as usize;
    let segment_size = read_u32(&program_header[0x10..0x10+4]) as usize;

    if virtual_offset + segment_size < virtual_offset {
        return Err("Segment size overflow error");
    }

    let virtual_segment = virtual_offset..virtual_offset + segment_size;

    put!("Checking:", entry_point as i32, file_offset as i32, virtual_offset as i32, segment_size as i32);

    if virtual_segment.contains(&entry_point) {
        Ok(Some(entry_point + file_offset - virtual_offset))
    } else {
        Ok(None)
    }
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([
        bytes[0],
        bytes[1],
    ])
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
    ])
}