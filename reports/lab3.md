##  编程作业

1. 将`sys_get_time`,`sys_mmap`,`sys_munmap`迁移至当前进程结构
2. 实现`spawn`系统调用：参考`fork`和`exec`实现了`spawn`功能
3. 实现`stride`调度算法：参考[OS的调度基础 [三]](https://zhuanlan.zhihu.com/p/124313667)，并使用Microsoft Copilot辅助理解该算法。
   在`TaskControlBlock`中添加了`stride`和`priority`以记录每个任务的优先级与步长，在`fetch`时使用stride调度算法，调用下一个执行的任务。

## 问答题

1. 不是，因为`p2.stride` 会溢出变成4，导致`p2.stride` <` p1.stride`
2. 每次执行都是选择`stride`最小的进程执行，而`pass`最大值为BigStride/2，所以*STRIDE_MAX – STRIDE_MIN <= BigStride / 2*

3. ```Rust
   use core::cmp::Ordering;
   
   struct Stride(u64);
   
   impl PartialOrd for Stride {
       fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
           let diff = self.0.wrapping_sub(other.0) as u8;
           if diff >= 128 {
               Some(Ordering::Less)
           } else {
               Some(Ordering::Greater)
           }
       }
   }
   
   impl PartialEq for Stride {
       fn eq(&self, other: &Self) -> bool {
           false
       }
   }
   ```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > *无*

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > [OS的调度基础 [三]](https://zhuanlan.zhihu.com/p/124313667)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。