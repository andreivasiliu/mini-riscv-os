use core::mem::MaybeUninit;

use crate::{
    bios_interface::{flash_page_erase, flash_write, get_char},
    put,
};

const FS_PREFIX: &[u8] = b"[\x1b[1;34mfs\x1b[0m]";

#[repr(C, align(1024))]
pub(crate) struct FileSystem {
    block_info: [BlockInfo; 64],
    free_blocks: Stack,
    first_blocks: Stack,
    blocks: *const [Block; 64],
    initialized: bool,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct BlockInfo {
    next_block: BlockId,
    file_name_size: u8,
    content_size: u16,
}

#[repr(C)]
struct Stack {
    elements: [BlockId; 64],
    count: u8,
}

#[repr(C, align(1024))]
struct Block {
    bytes: [u8; 1024],
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct BlockId(u8);

// Lives at 0x20000000 + 16 * 1024
// Saves into 0x08000000 + 63 * 1024
// Handles 64 blocks at 0x08000000 + 64 * 1024
impl FileSystem {
    pub(crate) fn new_from_scratch() -> &'static mut Self {
        let address = (0x20000000 + 16 * 1024) as *mut u8;
        let file_system: &mut MaybeUninit<FileSystem> =
            unsafe { (address as *mut MaybeUninit<FileSystem>).as_mut().unwrap() };

        let file_system = file_system.write(FileSystem {
            block_info: [BlockInfo {
                next_block: BlockId(0),
                file_name_size: 0,
                content_size: 0,
            }; 64],
            free_blocks: Stack {
                elements: [BlockId(0); 64],
                count: 0,
            },
            first_blocks: Stack {
                elements: [BlockId(0); 64],
                count: 0,
            },
            blocks: (0x08000000 + 64 * 1024) as *const [Block; 64],
            initialized: true,
        });

        for block in 0..64 {
            file_system.free_blocks.push(BlockId(block));
        }

        file_system
    }

    pub(crate) fn save_file_system(&self) {
        let source_page = 16;
        let target_page = 63;
        self.save(source_page, target_page);
    }

    pub(crate) fn save_block(&self, block_id: BlockId) {
        let source_page = 0;
        let target_page = block_id.0 + 64;
        self.save(source_page, target_page);
    }

    fn save(&self, source_page: u8, target_page: u8) {
        put!(FS_PREFIX, "Erasing flash page", target_page as u32 as i32);
        flash_page_erase(target_page);
        put!(
            FS_PREFIX,
            "Writing flash page from block",
            source_page as u32 as i32
        );
        flash_write(source_page, target_page);
        FileSystem::check();
    }

    pub(crate) fn load_from_flash() -> &'static mut Self {
        let src_address = (0x08000000 + 63 * 1024) as *mut u32;
        let dst_address = (0x20000000 + 16 * 1024) as *mut u32;

        for index in 0..256 {
            unsafe {
                let value = src_address.offset(index).read_volatile();
                dst_address.offset(index).write_volatile(value);
            }
        }

        unsafe { (dst_address as *mut FileSystem).as_mut().unwrap() }
    }

    pub(crate) fn create_file(&mut self, file_name: &[u8], content: &[u8]) -> Option<BlockId> {
        put!(
            FS_PREFIX,
            "Creating file",
            file_name,
            "with size",
            content.len() as i32
        );

        let free_block = self.free_blocks.pop()?;
        self.first_blocks.push(free_block);
        let file_name = &file_name[..256.min(file_name.len())];

        if free_block.0 > 63 {
            return None;
        }
        let block_info = &mut self.block_info[free_block.0 as usize];
        block_info.file_name_size = file_name.len() as u8;
        block_info.content_size = content.len() as u16;
        block_info.next_block = free_block;

        let write_block = (0x20000000 + 0 * 1024) as *mut u8;
        for index in 0..file_name.len() {
            unsafe {
                write_block
                    .offset(index as isize)
                    .write_volatile(file_name[index]);
            }
        }
        if file_name.len() + content.len() >= 1024 {
            // TODO: write to next block
            return None;
        }
        for index in 0..content.len() {
            unsafe {
                write_block
                    .offset((file_name.len() + index) as isize)
                    .write_volatile(content[index]);
            }
        }
        self.save_file_system();
        self.save_block(free_block);

        Some(free_block)
    }

    pub(crate) fn remove_file(&mut self, file: BlockId) {
        put!(FS_PREFIX, "Deleting file with block", file.0 as i32);
        let mut current_block = file;
        self.first_blocks.remove(file);

        loop {
            put!(
                FS_PREFIX,
                "Reclaiming content block",
                current_block.0 as i32
            );
            self.free_blocks.push(current_block);

            let block_info = self.block_info[current_block.0 as usize % 64];

            if block_info.next_block.0 == current_block.0 {
                break;
            }

            current_block = block_info.next_block;
        }
    }

    pub(crate) fn paste_file(&mut self, file_name: &[u8], file_size: usize) {
        put!("Pasting", file_size as i32, "bytes into:", file_name);

        let mut content = [0; 1024];

        if file_size >= 1024 {
            put!("File size too large, multiple blocks not yet supported.");
            return;
        }

        for index in 0..file_size {
            content[index % 1024] = get_char();
        }

        put!("Done.");

        if let Some(file) = self.file(file_name) {
            put!("Removing existing file");
            self.remove_file(file);
        }
        self.create_file(file_name, &content[0..file_size % 1024]);
    }

    pub(crate) fn list_files(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.first_blocks.iter()
    }

    pub(crate) fn print_stats(&self) {
        let initialized = if self.initialized {
            b"yes".as_slice()
        } else {
            b"no"
        };
        put!("Initialized:", initialized);
        put!("File blocks: ", self.first_blocks.count as u32 as i32);
        put!("Free blocks: ", self.free_blocks.count as u32 as i32);
    }

    pub(crate) fn file_name(&self, block_id: BlockId) -> &[u8] {
        let block_info = &self.block_info[block_id.0 as usize % 64];
        let block = unsafe { &(*self.blocks)[block_id.0 as usize % 64] };

        &block.bytes[..block_info.file_name_size as usize % 256]
    }

    pub(crate) fn file(&self, file_name: &[u8]) -> Option<BlockId> {
        for file in self.list_files() {
            let file_name_2 = self.file_name(file);
            if file_name == file_name_2 {
                return Some(file);
            }
        }

        None
    }

    fn block(&self, block_id: BlockId) -> &BlockInfo {
        &self.block_info[block_id.0 as usize % 64]
    }

    fn bytes(&self, block_id: BlockId) -> &[u8; 1024] {
        let block = unsafe { &(*self.blocks)[block_id.0 as usize % 64] };
        &block.bytes
    }

    #[inline(never)]
    pub(crate) fn read_file(&self, block_id: BlockId) -> &[u8] {
        let bytes = self.bytes(block_id);
        let block_info = self.block(block_id);

        let start = block_info.file_name_size as usize % 1024;
        let end = (start + block_info.content_size as usize) % 1024;

        let end = if start > end { start } else { end };

        let content = &bytes[start..end];

        content
    }

    #[inline(never)]
    pub(crate) fn check() {
        let wp0 = 0x1ffff808 as *const i32;
        let wp2 = 0x1ffff80c as *const i32;
        let wp0 = unsafe { wp0.read() };
        let wp2 = unsafe { wp2.read() };

        let status = unsafe { (0x4002200C as *const i32).read_volatile() };

        put!(
            FS_PREFIX,
            "wp0",
            wp0,
            "wp2",
            wp2,
            "busy:",
            status & (1 << 0),
            "pgerr:",
            status & (1 << 2),
            "wperr:",
            status & (1 << 4),
            "endf:",
            status & (1 << 5)
        );
    }
}

impl Stack {
    fn pop(&mut self) -> Option<BlockId> {
        if self.count == 0 || self.count > 64 {
            return None;
        }

        self.count -= 1;
        Some(self.elements[self.count as usize])
    }

    fn push(&mut self, block_id: BlockId) {
        if self.count > 63 {
            return;
        }
        self.elements[self.count as usize] = block_id;

        self.count += 1;
    }

    fn remove(&mut self, block_id: BlockId) {
        for (index, block_id_element) in self.elements.iter().enumerate() {
            if block_id.0 == block_id_element.0 {
                self.count -= 1;
                self.elements[index % 64] = self.elements[self.count as usize % 64];
                break;
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = BlockId> + '_ {
        (0..self.count)
            .into_iter()
            .map(|index| self.elements[index as usize % 64])
    }
}
