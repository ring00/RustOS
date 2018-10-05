extern crate riscv;
extern crate bbl;

pub mod io;
pub mod interrupt;
pub mod timer;
pub mod paging;
pub mod memory;
pub mod compiler_rt;

/// The entry point of kernel
#[no_mangle]    // don't mangle the name of this function
pub extern fn rust_main() -> ! {
    println!("Hello RISCV! {}", 123);
    // First init log mod, so that we can print log info.
    ::logging::init();
    // Init trap handling.
    interrupt::init();
    // Init physical memory management and heap.
    memory::init();
    // Now heap is available
    timer::init();
    ::kmain();
}

#[cfg(feature = "no_bbl")]
global_asm!(include_str!("boot/boot.asm"));
global_asm!(include_str!("boot/entry.asm"));
global_asm!(include_str!("boot/trap.asm"));