//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;
pub use alloc::vec;
pub use crate::task::{current_process};

/// deadlock checker
pub fn deadlock_checker() -> bool{
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let threads_num = process_inner.tasks.len();

    let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
    let mut work = vec![0; resource_types];
    for i in 0..threads_num {
        if let Some(task) = &process_inner.tasks[i] {
            let guard = task.inner_exclusive_access();
            if let Some(res) = guard.res.as_ref() {
                let tid = res.tid;
                if tid < process_inner.available.len() {
                    for j in 0..resource_types {
                        if j < process_inner.available[tid].len() {
                            work[j] += process_inner.available[tid][j];
                        }
                    }
                }
            }
        }
    }

    let mut finish = vec![false; threads_num];
    let need = &process_inner.need;
    let allocation = &process_inner.allocation;

    loop {
        let mut found = false;
        for i in 0..threads_num {
            if let Some(task) = &process_inner.tasks[i] {
                let guard = task.inner_exclusive_access();
                if let Some(res) = guard.res.as_ref() {
                    let tid = res.tid;
                    if !finish[i] && tid < need.len() && tid < allocation.len() {
                        let rnum = work.len();
                        let mut can_finish = true;
                        for r in 0..rnum {
                            let need_val = if r < need[tid].len() { need[tid][r] } else { 0 };
                            if need_val > work[r] {
                                can_finish = false;
                                break;
                            }
                        }
                        if can_finish {
                            for r in 0..rnum {
                                let alloc_val = if r < allocation[tid].len() { allocation[tid][r] } else { 0 };
                                work[r] += alloc_val;
                            }
                            finish[i] = true;
                            found = true;
                        }
                    }
                } else {
                    finish[i] = true;
                }
            } else {
                finish[i] = true;
            }
        }
        if !found {
            break;
        }
    }

    finish.iter().all(|&f| f)
}
