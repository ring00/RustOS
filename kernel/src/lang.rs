/**
 * @file lang.rs
 * @brief Rust language features implementions.
 */

use core::panic::PanicInfo;
use core::alloc::Layout;

#[lang = "eh_personality"]
extern fn eh_personality() {
}

/**
 * @brief Print error messages with location and loop forever.
 *
 * @param info PanicInfo
 */
#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    let message = info.message().unwrap();
    error!("\n\nPANIC in {} at line {}\n    {}", location.file(), location.line(), message);
    loop { }
}

/**
 * @brief Out of memory panic.
 */
#[lang = "oom"]
#[no_mangle]
pub fn oom(_: Layout) -> ! {
    panic!("out of memory");
}
