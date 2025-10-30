# ch6

## 编程作业

1. 将`sys_get_time`,`sys_mmap`,`sys_munmap`,`sys_spawn`,`sys_set_priority`迁移至本实验中，同时修改了`sys_spawn`,使其能正确找到程序
2. 实现了`sys_linkat`，将文件路径与一个已有的文件进行硬链接，同时能处理同名文件等错误
3. 修改了`Inode.find`和`Inode.create`,使其返回对应的`Inode_id`
4. 实现了`sys_unlinkat`,将文件路径与对应的文件取消链接，同时能处理文件不存在等错误，在彻底删除时释放数据块
5. 实现了`sys_fstat`，为`File Trait`增加stat接口，以获取对应文件的状态

## 问答作业

root inode在easy-fs中是作为根目录，记录所有目录项；如果损坏，文件系统将不可用，无法访问任何文件。

# ch7

