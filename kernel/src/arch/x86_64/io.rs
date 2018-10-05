use super::driver::serial::*;
use core::fmt::{Arguments, Write};

/* *
 * getchar
 * @brief:  get a char from serial port
 * @param:  none
 * @retval: char
 * */
pub fn getchar() -> char {
    unsafe { COM1.force_unlock(); }
    COM1.lock().receive() as char
}

/* *
 * putfmt
 * @brief:  output fmt into serial port
 * @param:
    fmt:    output arguments(core::fmt::Arguments), including string to output
 * @retval: none
 * */
pub fn putfmt(fmt: Arguments) {
    unsafe { COM1.force_unlock(); }
    COM1.lock().write_fmt(fmt).unwrap()
}