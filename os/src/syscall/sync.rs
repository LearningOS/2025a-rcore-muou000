use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore, deadlock_checker};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    // 获取创建者线程 tid，用于把初始可用资源集中放在该行
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if process_inner.enable_deadlock_detect {
        let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len() + 1;
        let max_tid = process_inner.tasks.len();
        while process_inner.available.len() < max_tid {
            process_inner.available.push(vec![]);
        }
        while process_inner.allocation.len() < max_tid {
            process_inner.allocation.push(vec![]);
        }
        while process_inner.need.len() < max_tid {
            process_inner.need.push(vec![]);
        }
        for thread_id in 0..max_tid {
            while process_inner.available[thread_id].len() < resource_types {
                process_inner.available[thread_id].push(0);
            }
            while process_inner.allocation[thread_id].len() < resource_types {
                process_inner.allocation[thread_id].push(0);
            }
            while process_inner.need[thread_id].len() < resource_types {
                process_inner.need[thread_id].push(0);
            }
        }
    }
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
    if process_inner.enable_deadlock_detect {
            for thread_id in 0..process_inner.tasks.len() {
                process_inner.available[thread_id][id] = if thread_id == tid { 1 } else { 0 };
                process_inner.allocation[thread_id][id] = 0;
                process_inner.need[thread_id][id] = 0;
            }
        }
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        let id = process_inner.mutex_list.len() - 1;
        if process_inner.enable_deadlock_detect {
            for thread_id in 0..process_inner.tasks.len() {
                process_inner.available[thread_id][id] = if thread_id == tid { 1 } else { 0 };
                process_inner.allocation[thread_id][id] = 0;
                process_inner.need[thread_id][id] = 0;
            }
        }
        id as isize
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let enable_deadlock_detect = {
        let process_inner = process.inner_exclusive_access();
        process_inner.enable_deadlock_detect
    };

    if enable_deadlock_detect {
        {
            let mut process_inner = process.inner_exclusive_access();
            let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
            while process_inner.available.len() <= tid {
                process_inner.available.push(vec![]);
            }
            while process_inner.allocation.len() <= tid {
                process_inner.allocation.push(vec![]);
            }
            while process_inner.need.len() <= tid {
                process_inner.need.push(vec![]);
            }
            while process_inner.available[tid].len() < resource_types {
                process_inner.available[tid].push(0);
            }
            while process_inner.allocation[tid].len() < resource_types {
                process_inner.allocation[tid].push(0);
            }
            while process_inner.need[tid].len() < resource_types {
                process_inner.need[tid].push(0);
            }
            if process_inner.need[tid][mutex_id] == 0 {
                process_inner.need[tid][mutex_id] = 1;
            }

            if process_inner.available[tid][mutex_id] <= 0 {
                return -0xdead;
            }
            if process_inner.need[tid][mutex_id] <= 0 {
                return -0xdead;
            }

            process_inner.available[tid][mutex_id] -= 1;
            process_inner.allocation[tid][mutex_id] += 1;
            process_inner.need[tid][mutex_id] -= 1;
        }

        if !deadlock_checker() {
            let mut process_inner = process.inner_exclusive_access();
            process_inner.available[tid][mutex_id] += 1;
            process_inner.allocation[tid][mutex_id] -= 1;
            process_inner.need[tid][mutex_id] += 1;
            return -0xdead;
        }
    }

    let mutex = {
        let process_inner = process.inner_exclusive_access();
        Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap())
    };
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let enable_deadlock_detect = {
        let process_inner = process.inner_exclusive_access();
        process_inner.enable_deadlock_detect
    };

    if enable_deadlock_detect {
        let mut process_inner = process.inner_exclusive_access();
        process_inner.available[tid][mutex_id] += 1;
        process_inner.allocation[tid][mutex_id] -= 1;
        process_inner.need[tid][mutex_id] += 1;
    }

    let mutex = {
        let process_inner = process.inner_exclusive_access();
        Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap())
    };
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let mutex_len = process_inner.mutex_list.len();
    if process_inner.enable_deadlock_detect {
        while process_inner.available.len() <= tid {
            process_inner.available.push(vec![]);
        }
        while process_inner.allocation.len() <= tid {
            process_inner.allocation.push(vec![]);
        }
        while process_inner.need.len() <= tid {
            process_inner.need.push(vec![]);
        }
    }
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        if process_inner.enable_deadlock_detect {
            let mut thread_ids = vec![];
            for t in 0..process_inner.tasks.len() {
                if let Some(task) = &process_inner.tasks[t] {
                    if let Some(res) = task.inner_exclusive_access().res.as_ref() {
                        thread_ids.push(res.tid);
                    }
                }
            }
            for thread_id in thread_ids {
                while process_inner.available.len() <= thread_id {
                    process_inner.available.push(vec![]);
                }
                while process_inner.allocation.len() <= thread_id {
                    process_inner.allocation.push(vec![]);
                }
                while process_inner.need.len() <= thread_id {
                    process_inner.need.push(vec![]);
                }
                let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
                while process_inner.available[thread_id].len() < resource_types {
                    process_inner.available[thread_id].push(0);
                }
                while process_inner.allocation[thread_id].len() < resource_types {
                    process_inner.allocation[thread_id].push(0);
                }
                while process_inner.need[thread_id].len() < resource_types {
                    process_inner.need[thread_id].push(0);
                }
                let gid = mutex_len + id;
                process_inner.available[thread_id][gid] = if thread_id == tid { res_count } else { 0 };
                process_inner.allocation[thread_id][gid] = 0;
                process_inner.need[thread_id][gid] = 0;
            }
        }
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        if process_inner.enable_deadlock_detect {
            let mut thread_ids = vec![];
            for t in 0..process_inner.tasks.len() {
                if let Some(task) = &process_inner.tasks[t] {
                    if let Some(res) = task.inner_exclusive_access().res.as_ref() {
                        thread_ids.push(res.tid);
                    }
                }
            }

            let gid = mutex_len + (process_inner.semaphore_list.len() - 1);
            let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
            for thread_id in thread_ids {
                while process_inner.available.len() <= thread_id {
                    process_inner.available.push(vec![]);
                }
                while process_inner.allocation.len() <= thread_id {
                    process_inner.allocation.push(vec![]);
                }
                while process_inner.need.len() <= thread_id {
                    process_inner.need.push(vec![]);
                }
                while process_inner.available[thread_id].len() < resource_types {
                    process_inner.available[thread_id].push(0);
                }
                while process_inner.allocation[thread_id].len() < resource_types {
                    process_inner.allocation[thread_id].push(0);
                }
                while process_inner.need[thread_id].len() < resource_types {
                    process_inner.need[thread_id].push(0);
                }
                process_inner.available[thread_id][gid] = if thread_id == tid { res_count } else { 0 };
                process_inner.allocation[thread_id][gid] = 0;
                process_inner.need[thread_id][gid] = 0;
            }
        }
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    if process.inner_exclusive_access().enable_deadlock_detect {
        let mut process_inner = process.inner_exclusive_access();
        let gid = process_inner.mutex_list.len() + sem_id;
        process_inner.available[tid][gid] += 1;
        process_inner.allocation[tid][gid] -= 1;
    }

    let sem = {
        let process_inner = process.inner_exclusive_access();
        Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap())
    };
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let mut pre_allocated = false;
    let mut donor_tid: Option<usize> = None;
    if process.inner_exclusive_access().enable_deadlock_detect {
        {
            let mut process_inner = process.inner_exclusive_access();
            let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
            while process_inner.available.len() <= tid {
                process_inner.available.push(vec![]);
            }
            while process_inner.allocation.len() <= tid {
                process_inner.allocation.push(vec![]);
            }
            while process_inner.need.len() <= tid {
                process_inner.need.push(vec![]);
            }
            while process_inner.available[tid].len() < resource_types {
                process_inner.available[tid].push(0);
            }
            while process_inner.allocation[tid].len() < resource_types {
                process_inner.allocation[tid].push(0);
            }
            while process_inner.need[tid].len() < resource_types {
                process_inner.need[tid].push(0);
            }

            let gid = process_inner.mutex_list.len() + sem_id;

            if process_inner.need[tid][gid] == 0 {
                process_inner.need[tid][gid] = 1;
            }

            let mut total_available = 0isize;
            for row in 0..process_inner.available.len() {
                if process_inner.available[row].len() > gid {
                    total_available += process_inner.available[row][gid] as isize;
                    if donor_tid.is_none() && process_inner.available[row][gid] > 0 {
                        donor_tid = Some(row);
                    }
                }
            }

            if total_available > 0 {
                let from = donor_tid.unwrap_or(tid);
                if process_inner.available[from].len() <= gid {
                    return -0xdead;
                }
                process_inner.available[from][gid] -= 1;
                process_inner.allocation[tid][gid] += 1;
                process_inner.need[tid][gid] -= 1;
                pre_allocated = true;
            } else {
            }
        }

        if !deadlock_checker() {
            let mut process_inner = process.inner_exclusive_access();
            let gid = process_inner.mutex_list.len() + sem_id;
            if pre_allocated {
                let from = donor_tid.unwrap_or(tid);
                process_inner.available[from][gid] += 1;
                process_inner.allocation[tid][gid] -= 1;
                process_inner.need[tid][gid] += 1;
            }
            return -0xdead;
        }
    }

    let sem = {
        let process_inner = process.inner_exclusive_access();
        Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap())
    };
    sem.down();

    if process.inner_exclusive_access().enable_deadlock_detect && !pre_allocated {
        let mut process_inner = process.inner_exclusive_access();
        let gid = process_inner.mutex_list.len() + sem_id;
        let mut from_opt: Option<usize> = None;
        for row in 0..process_inner.available.len() {
            if process_inner.available[row].len() > gid && process_inner.available[row][gid] > 0 {
                from_opt = Some(row);
                break;
            }
        }
        let from = from_opt.unwrap_or(tid);
        if process_inner.available[from].len() > gid {
            process_inner.available[from][gid] -= 1;
        }
        process_inner.allocation[tid][gid] += 1;
        if process_inner.need[tid][gid] > 0 { process_inner.need[tid][gid] -= 1; }
    }
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    match enabled {
        0 => {
            process_inner.enable_deadlock_detect = false;
            return 0;
        }
        1 => {
            process_inner.enable_deadlock_detect = true;
            let max_tid = process_inner.tasks.len();
            while process_inner.available.len() < max_tid {
                process_inner.available.push(vec![]);
            }
            while process_inner.allocation.len() < max_tid {
                process_inner.allocation.push(vec![]);
            }
            while process_inner.need.len() < max_tid {
                process_inner.need.push(vec![]);
            }
            let resource_types = process_inner.mutex_list.len() + process_inner.semaphore_list.len();
            for thread_id in 0..max_tid {
                while process_inner.available[thread_id].len() < resource_types {
                    process_inner.available[thread_id].push(0);
                }
                while process_inner.allocation[thread_id].len() < resource_types {
                    process_inner.allocation[thread_id].push(0);
                }
                while process_inner.need[thread_id].len() < resource_types {
                    process_inner.need[thread_id].push(0);
                }
            }
            return 0;
        }
        _ => {
            return -1;
        }
    }
}
