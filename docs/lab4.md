# 内核线程管理

## 实验目的

* 了解内核线程创建和执行的管理过程
* 了解内核线程的切换和基本调度过程

## 实验内容

我们之前已经完成了物理内存管理和虚拟内存管理的实验。
除了内存管理外，内核还需要考虑如何分时使用处理器来并发执行多个进程，本实验将介绍相关关原理。

## 实验原理

程序等于算法加数据结构，为了实现内核线程管理的功能，我们一方面需要表示内核线程的数据结构，另一方面需要在线程间进行切换的调度算法。

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

### 线程切换

```rust
pub unsafe extern fn switch(&mut self, _target: &mut Self) {
    #[cfg(target_arch = "riscv32")]
    asm!(r"
    .equ XLENB, 4
    .macro Load reg, mem
        lw \reg, \mem
    .endm
    .macro Store reg, mem
        sw \reg, \mem
    .endm");
    asm!("
    // save from's registers
    addi  sp, sp, (-XLENB*14)
    Store sp, 0(a0)
    Store ra, 0*XLENB(sp)
    Store s0, 2*XLENB(sp)
    Store s1, 3*XLENB(sp)
    Store s2, 4*XLENB(sp)
    Store s3, 5*XLENB(sp)
    Store s4, 6*XLENB(sp)
    Store s5, 7*XLENB(sp)
    Store s6, 8*XLENB(sp)
    Store s7, 9*XLENB(sp)
    Store s8, 10*XLENB(sp)
    Store s9, 11*XLENB(sp)
    Store s10, 12*XLENB(sp)
    Store s11, 13*XLENB(sp)
    csrr  s11, satp
    Store s11, 1*XLENB(sp)
    // restore to's registers
    Load sp, 0(a1)
    Load s11, 1*XLENB(sp)
    csrw satp, s11
    Load ra, 0*XLENB(sp)
    Load s0, 2*XLENB(sp)
    Load s1, 3*XLENB(sp)
    Load s2, 4*XLENB(sp)
    Load s3, 5*XLENB(sp)
    Load s4, 6*XLENB(sp)
    Load s5, 7*XLENB(sp)
    Load s6, 8*XLENB(sp)
    Load s7, 9*XLENB(sp)
    Load s8, 10*XLENB(sp)
    Load s9, 11*XLENB(sp)
    Load s10, 12*XLENB(sp)
    Load s11, 13*XLENB(sp)
    addi sp, sp, (XLENB*14)
    Store zero, 0(a1)
    ret"
    : : : : "volatile" )
}
```

### 线程创建

```rust
fn new_kernel_thread(entry: extern fn(usize) -> !, arg: usize, sp: usize) -> Self {
    use core::mem::zeroed;
    let mut tf: Self = unsafe { zeroed() };
    tf.x[10] = arg; // a0
    tf.x[2] = sp;
    tf.sepc = entry as usize;
    tf.sstatus = xstatus::read();
    tf.sstatus.set_xpie(true);
    tf.sstatus.set_xie(false);
    #[cfg(feature = "m_mode")]
    tf.sstatus.set_mpp(xstatus::MPP::Machine);
    #[cfg(not(feature = "m_mode"))]
    tf.sstatus.set_spp(xstatus::SPP::Supervisor);
    tf
}
```
