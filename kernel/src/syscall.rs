/**
 * @file syscall.rs
 * @brief 系统调用解析执行模块
 */

#![allow(unused)]

use arch::interrupt::TrapFrame;
use process::*;
use thread;
use util;

/**
 * @brief 系统调用入口点
 *
 * 当发生系统调用中断时，中断服务例程将控制权转移到这里。
 *
 * @param id Syscall id.
 * @param args Syscall arguments.
 * @param tf Current process's trap frame.
 * @retval i32 Syscall return value.
 */
pub fn syscall(id: usize, args: [usize; 6], tf: &TrapFrame) -> i32 {
    match id {
        SYS_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_OPEN => sys_open(args[0] as *const u8, args[1]),
        SYS_CLOSE => sys_close(args[0]),
        SYS_WAIT => sys_wait(args[0], args[1] as *mut i32),
        SYS_FORK => sys_fork(tf),
        SYS_KILL => sys_kill(args[0]),
        SYS_EXIT => sys_exit(args[0]),
        SYS_YIELD => sys_yield(),
        SYS_GETPID => sys_getpid(),
        SYS_SLEEP => sys_sleep(args[0]),
        SYS_GETTIME => sys_get_time(),
        SYS_LAB6_SET_PRIORITY => sys_lab6_set_priority(args[0]),
        SYS_PUTC => sys_putc(args[0] as u8 as char),
        _ => {
            error!("unknown syscall id: {:#x?}, args: {:x?}", id, args);
            ::trap::error(tf);
        }
    }
}

/**
 * @brief Write data into a file.
 *
 * @param fd File descriptor
 * @param base Data address to write.
 * @param len Number of bytes to write.
 * @retval i32 Always 0.
 */
fn sys_write(fd: usize, base: *const u8, len: usize) -> i32 {
    info!("write: fd: {}, base: {:?}, len: {:#x}", fd, base, len);
    use core::slice;
    use core::str;
    let slice = unsafe { slice::from_raw_parts(base, len) };
    print!("{}", str::from_utf8(slice).unwrap());
    0
}

/**
 * @brief Open a file.
 *
 * @param path File path.
 * @param flags Open flags.
 * @retval i32 `stdin`: 0; `stdout`: 1; others: -1;
 */
fn sys_open(path: *const u8, flags: usize) -> i32 {
    let path = unsafe { util::from_cstr(path) };
    info!("open: path: {:?}, flags: {:?}", path, flags);
    match path {
        "stdin:" => 0,
        "stdout:" => 1,
        _ => -1,
    }
}

/**
 * @brief Close an opened file.
 *
 * @param fd File descriptor.
 * @retval i32 Always 0.
 */
fn sys_close(fd: usize) -> i32 {
    info!("close: fd: {:?}", fd);
    0
}

/**
 * @brief Fork the current process.
 *
 * @param tf Current process's trap frame.
 * @retval i32 The child's PID.
 */
fn sys_fork(tf: &TrapFrame) -> i32 {
    let mut processor = processor();
    let context = processor.current_context().fork(tf);
    let pid = processor.add(context);
    info!("fork: {} -> {}", processor.current_pid(), pid);
    pid as i32
}

/**
 * @brief Wait the process exit.
 *
 * @param pid The process to wait.
 * @param code Store exit code to `code` if it's not null.
 * @retval i32 The PID.
 */
fn sys_wait(pid: usize, code: *mut i32) -> i32 {
    let mut processor = processor();
    match processor.current_wait_for(pid) {
        WaitResult::Ok(pid, error_code) => {
            if !code.is_null() {
                unsafe { *code = error_code as i32 };
            }
            0
        }
        WaitResult::NotExist => -1,
    }
}

/**
 * @brief Wait the process exit.
 *
 * @retval i32 Always 0.
 */
fn sys_yield() -> i32 {
    thread::yield_now();
    0
}

/**
 * @brief Kill the process.
 *
 * @param pid The process to kill.
 * @retval i32 Always 0.
 */
fn sys_kill(pid: usize) -> i32 {
    processor().kill(pid);
    0
}

/**
 * @brief Get the current process id.
 *
 * @retval i32 PID.
 */
fn sys_getpid() -> i32 {
    thread::current().id() as i32
}

/**
 * @brief Exit the current process.
 *
 * @param error_code Exit with this code.
 * @retval i32 Always 0.
 */
fn sys_exit(error_code: usize) -> i32 {
    let mut processor = processor();
    let pid = processor.current_pid();
    processor.exit(pid, error_code);
    0
}

/**
 * @brief Current process sleep some time.
 *
 * @param time Duration(millisecond).
 * @retval i32 Always 0.
 */
fn sys_sleep(time: usize) -> i32 {
    use core::time::Duration;
    thread::sleep(Duration::from_millis(time as u64 * 10));
    0
}

/**
 * @brief Get current system time.
 *
 * @retval i32 The time.
 */
fn sys_get_time() -> i32 {
    let processor = processor();
    processor.get_time() as i32
}

/**
 * @brief Set process priority for lab6.
 *
 * @param priority The priority.
 * @retval i32 Always 0.
 */
fn sys_lab6_set_priority(priority: usize) -> i32 {
    let mut processor = processor();
    processor.set_priority(priority as u8);
    0
}

/**
 * @brief Print a char.
 *
 * @retval i32 Always 0.
 */
fn sys_putc(c: char) -> i32 {
    print!("{}", c);
    0
}

const SYS_EXIT: usize = 1;
const SYS_FORK: usize = 2;
const SYS_WAIT: usize = 3;
const SYS_EXEC: usize = 4;
const SYS_CLONE: usize = 5;
const SYS_YIELD: usize = 10;
const SYS_SLEEP: usize = 11;
const SYS_KILL: usize = 12;
const SYS_GETTIME: usize = 17;
const SYS_GETPID: usize = 18;
const SYS_MMAP: usize = 20;
const SYS_MUNMAP: usize = 21;
const SYS_SHMEM: usize = 22;
const SYS_PUTC: usize = 30;
const SYS_PGDIR: usize = 31;
const SYS_OPEN: usize = 100;
const SYS_CLOSE: usize = 101;
const SYS_READ: usize = 102;
const SYS_WRITE: usize = 103;
const SYS_SEEK: usize = 104;
const SYS_FSTAT: usize = 110;
const SYS_FSYNC: usize = 111;
const SYS_GETCWD: usize = 121;
const SYS_GETDIRENTRY: usize = 128;
const SYS_DUP: usize = 130;
const SYS_LAB6_SET_PRIORITY: usize = 255;
