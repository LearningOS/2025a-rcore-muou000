//! Process management syscalls

use crate::task::{change_program_brk, exit_current_and_run_next, get_call_times, mmap_area, munmap_area, suspend_current_and_run_next};
use crate::mm::{is_readable, is_writable, is_user_accessable, translate_ptr, VirtAddr};
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let phys_ptr: *mut TimeVal = translate_ptr(ts);
    let us = crate::timer::get_time_us();
    unsafe {
        *phys_ptr = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");

    match trace_request {
        0 => {
            if !is_readable(id) || !is_user_accessable(id) {
                return -1;
            }
            let phys_ptr = translate_ptr(id as *const u8);
            let ptr = phys_ptr as *const u8;
            unsafe {
                core::ptr::read_volatile(ptr) as isize
            }
        }
        1 => {
            if !is_writable(id) || !is_user_accessable(id) {
                return -1;
            }
            let phys_ptr = translate_ptr(id as *mut u8);
            let ptr = phys_ptr as *mut u8;
            unsafe {
                core::ptr::write_volatile(ptr, data as u8);
            }
            0
        }
        2 => {
            get_call_times(id) as isize
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % 4096 != 0 || port & !0x7 != 0 || port == 0 || len == 0 {
        return -1;
    }
    if mmap_area(start, len, port) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let va = VirtAddr::from(start);
    if va.page_offset() != 0 || len % 4096 != 0 || len == 0{
        return -1;
    }
    munmap_area(start, len);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
