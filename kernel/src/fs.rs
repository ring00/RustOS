/**
 * @file fs.rs
 * @brief File system.
 */

use simple_filesystem::*;
use alloc::boxed::Box;
#[cfg(target_arch = "x86_64")]
use arch::driver::ide;
use spin::Mutex;

// Hard link user program.
#[cfg(target_arch = "riscv32")]
global_asm!(r#"
    .section .rodata
    .align 12
_binary_user_riscv_img_start:
    .incbin "../user/user-riscv.img"
_binary_user_riscv_img_end:
"#);

/**
 * @brief Startup the shell.
 *
 * Load fs from ide::DISK1, waiting for input commands and execute it.
 *
 */
pub fn shell() {
    #[cfg(target_arch = "riscv32")]
    let device = {
        extern {
            fn _binary_user_riscv_img_start();
            fn _binary_user_riscv_img_end();
        }
        Box::new(unsafe { MemBuf::new(_binary_user_riscv_img_start, _binary_user_riscv_img_end) })
    };
    #[cfg(target_arch = "x86_64")]
    let device = Box::new(&ide::DISK1);
    let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
    let root = sfs.root_inode();
    let files = root.borrow().list().unwrap();
    println!("Available programs: {:?}", files);

    // Avoid stack overflow in release mode
    // Equal to: `buf = Box::new([0; 64 << 12])`
    use alloc::alloc::{alloc, dealloc, Layout};
    const BUF_SIZE: usize = 0x40000;
    let layout = Layout::from_size_align(BUF_SIZE, 0x1000).unwrap();
    let buf = unsafe{ slice::from_raw_parts_mut(alloc(layout), BUF_SIZE) };
    loop {
        print!(">> ");
        use console::get_line;
        let name = get_line();
        if name == "" {
            continue;
        }
        if let Ok(file) = root.borrow().lookup(name.as_str()) {
            use process::*;
            let len = file.borrow().read_at(0, &mut *buf).unwrap();
            let pid = processor().add(Context::new_user(&buf[..len]));
            processor().current_wait_for(pid);
        } else {
            println!("Program not exist");
        }
    }
    unsafe { dealloc(buf.as_mut_ptr(), layout) };
}

struct MemBuf(&'static [u8]);

impl MemBuf {
    /**
     * @brief Create a MemBuf from address `begin` to address `end`.
     *
     * @param begin
     * @param end
     * @retval MemBuf
     */
    unsafe fn new(begin: unsafe extern fn(), end: unsafe extern fn()) -> Self {
        use core::slice;
        MemBuf(slice::from_raw_parts(begin as *const u8, end as usize - begin as usize))
    }
}

impl Device for MemBuf {
    /**
     * @brief Read all data start at `offset` and store into `buf`.
     *
     * @param self The MemBuf to be read.
     * @param offset The position to read.
     * @param buf The array to store data.
     * @retval Option<usize> Number of bytes read.
     */
    fn read_at(&mut self, offset: usize, buf: &mut [u8]) -> Option<usize> {
        let slice = self.0;
        let len = buf.len().min(slice.len() - offset);
        buf[..len].copy_from_slice(&slice[offset..offset + len]);
        Some(len)
    }

    /**
     * @brief Write all data from `buf` and store into MemBuf with offset.
     *
     * @param self The MemBuf to write data.
     * @param offset The position at MemBuf to write.
     * @param buf The data to be written.
     * @retval none
     */
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> Option<usize> {
        None
    }
}

use core::slice;

#[cfg(target_arch = "x86_64")]
impl BlockedDevice for &'static ide::DISK1 {
    const BLOCK_SIZE_LOG2: u8 = 9;

    /**
     * @brief Read all data at the block `block_id` on ide::DSIK1.
     *
     * @param self ide::DISK1
     * @param block_id Block index.
     * @param buf The array to store data.
     * @retval bool Whether succeed.
     */
    fn read_at(&mut self, block_id: usize, buf: &mut [u8]) -> bool {
        assert!(buf.len() >= ide::BLOCK_SIZE);
        let buf = unsafe { slice::from_raw_parts_mut(buf.as_ptr() as *mut u32, ide::BLOCK_SIZE / 4) };
        self.0.lock().read(block_id as u64, 1, buf).is_ok()
    }

    /**
     * @brief Write all data into the block `block_id` on ide::DSIK1.
     *
     * @param self ide::DISK1
     * @param block_id Block index.
     * @param buf The data to be written.
     * @retval bool Whether succeed.
     */
    fn write_at(&mut self, block_id: usize, buf: &[u8]) -> bool {
        assert!(buf.len() >= ide::BLOCK_SIZE);
        let buf = unsafe { slice::from_raw_parts(buf.as_ptr() as *mut u32, ide::BLOCK_SIZE / 4) };
        self.0.lock().write(block_id as u64, 1, buf).is_ok()
    }
}
