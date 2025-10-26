//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
const BIG_STRIDE: usize = 1000000;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    /// 参考了知乎：OS的调度基础[三]https://zhuanlan.zhihu.com/p/124313667，并使用Microsoft Copilot辅助理解
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride = usize::MAX;
        let mut min_stride_index = 0;
        for (i, task_control_block) in self.ready_queue.iter().enumerate() {
            let inner = task_control_block.inner_exclusive_access();
            let current_stride = inner.stride;
            if current_stride < min_stride {
                min_stride = current_stride;
                min_stride_index = i;
            }
        }

        let task = self.ready_queue.remove(min_stride_index).unwrap();

        let mut inner = task.inner_exclusive_access();
        let pass = BIG_STRIDE / inner.priority;
        inner.stride += pass;
        drop(inner);

        Some(task)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
