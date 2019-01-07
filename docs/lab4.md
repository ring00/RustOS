# 内核线程管理

## 实验目的

* 了解内核线程创建和执行的管理过程
* 了解内核线程的切换和基本调度过程

## 实验内容

我们之前已经完成了物理内存管理和虚拟内存管理的实验。
除了内存管理外，内核还需要考虑如何分时使用处理器来并发执行多个进程，本实验将介绍相关关原理。

## 实验原理

程序等于算法加数据结构，为了实现内核线程管理的功能，我们一方面需要表示内核线程的数据结构，另一方面需要对线程进行管理的调度算法。

### 数据结构

#### 中断帧

发生中断时向内核栈顶部压入的数据结构，详情可参考实验一的文档。

```rust
pub struct TrapFrame {
    pub x: [usize; 32], // general registers
    pub sstatus: Xstatus, // Supervisor Status Register
    pub sepc: usize, // Supervisor exception program counter
    pub stval: usize, // Supervisor trap value
    pub scause: Mcause, // scause register: record the cause of exception/interrupt/trap
}
```

#### 线程上下文

一个线程必要的上下文信息，即callee-saved registers。

```rust
struct ContextData {
    ra: usize,
    satp: usize,
    s: [usize; 12],
}
```

#### 初始内核栈

一个新创建且未开始运行的线程对应的内核栈上除了保存寄存器外，还应当有手动构造的中断帧，以方便通过中断返回机制来执行新线程。

```rust
/// kernel stack contents for a new thread
pub struct InitStack {
    context: ContextData,
    tf: TrapFrame,
}
```

#### 进程控制块

进程需要实现的基本功能是从一个进程切换到另一个进程，可以用trait来表示。

```rust
pub trait Context {
    unsafe extern "C"
    fn switch_to(&mut self, target: &mut Context);
}
```

对应的实现如下所示。

```rust
use crate::arch::interrupt::Context as ArchContext;

pub struct Process {
    pub arch: ArchContext,
    pub memory_set: MemorySet,
    pub kstack: KernelStack,
    pub files: BTreeMap<usize, Arc<Mutex<File>>>,
    pub cwd: String,
}

impl Context for Process {
    unsafe fn switch_to(&mut self, target: &mut Context) {
        use core::mem::transmute;
        let (target, _): (&mut Process, *const ()) = transmute(target);
        self.arch.switch(&mut target.arch);
    }
}
```

### 线程创建

### 线程切换
