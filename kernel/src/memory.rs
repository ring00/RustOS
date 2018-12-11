pub use arch::paging::*;
use bit_allocator::{BitAlloc, BitAlloc4K, BitAlloc64K};
use consts::MEMORY_OFFSET;
use spin;
use super::HEAP_ALLOCATOR;
use ucore_memory::{*, paging::PageTable};
use ucore_memory::cow::CowExt;
pub use ucore_memory::memory_set::{MemoryArea, MemoryAttr, MemorySet as MemorySet_, InactivePageTable, MemoryHandler};
use ucore_memory::swap::{fifo, mock_swapper, SwapExt as SwapExt_};
//use process::{processor, PROCESSOR};
use process::{process};
use sync::{SpinNoIrqLock, SpinNoIrq, MutexGuard};
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::boxed::Box;
use core::slice;

pub type MemorySet = MemorySet_<InactivePageTable0>;
pub type SwapExtType = SwapExt_<fifo::FifoSwapManager, mock_swapper::MockSwapper, InactivePageTable0>;

// x86_64 support up to 256M memory
#[cfg(target_arch = "x86_64")]
pub type FrameAlloc = BitAlloc64K;

// RISCV only have 8M memory
#[cfg(target_arch = "riscv32")]
pub type FrameAlloc = BitAlloc4K;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SpinNoIrqLock<FrameAlloc> = SpinNoIrqLock::new(FrameAlloc::default());
}
// record the user memory set for pagefault function (swap in/out and frame delayed allocate) temporarily when page fault in new_user() or fork() function
// after the process is set we can use use processor() to get the inactive page table
/*
lazy_static! {
    pub static ref MEMORY_SET_RECORD: SpinNoIrqLock<VecDeque<usize>> = SpinNoIrqLock::new(VecDeque::default());
}

pub fn memory_set_record() -> MutexGuard<'static, VecDeque<usize>, SpinNoIrq> {
    MEMORY_SET_RECORD.lock()
}
*/

/*
lazy_static! {
    static ref ACTIVE_TABLE: SpinNoIrqLock<CowExt<ActivePageTable>> = SpinNoIrqLock::new(unsafe {
        CowExt::new(ActivePageTable::new())
    });
}

/// The only way to get active page table
pub fn active_table() -> MutexGuard<'static, CowExt<ActivePageTable>, SpinNoIrq> {
    ACTIVE_TABLE.lock()
}
*/

lazy_static! {
    static ref ACTIVE_TABLE: SpinNoIrqLock<ActivePageTable> = SpinNoIrqLock::new(unsafe {
        ActivePageTable::new()
    });
}

/// The only way to get active page table
pub fn active_table() -> MutexGuard<'static, ActivePageTable, SpinNoIrq> {
    ACTIVE_TABLE.lock()
}

/*
// Page table for swap in and out
lazy_static!{
    static ref ACTIVE_TABLE_SWAP: SpinNoIrqLock<SwapExt<ActivePageTable, fifo::FifoSwapManager, mock_swapper::MockSwapper>> =
        SpinNoIrqLock::new(unsafe{SwapExt::new(ActivePageTable::new(), fifo::FifoSwapManager::default(), mock_swapper::MockSwapper::default())});
}

pub fn active_table_swap() -> MutexGuard<'static, SwapExt<ActivePageTable, fifo::FifoSwapManager, mock_swapper::MockSwapper>, SpinNoIrq>{
    ACTIVE_TABLE_SWAP.lock()
}
*/

lazy_static!{
    pub static ref SWAP_TABLE: Arc<spin::Mutex<SwapExtType>> = 
        Arc::new(spin::Mutex::new(SwapExtType::new(fifo::FifoSwapManager::default(), mock_swapper::MockSwapper::default())));
}

pub fn swap_table() -> spin::MutexGuard<'static, SwapExtType>{
    SWAP_TABLE.lock()
}


lazy_static!{
    pub static ref COW_TABLE: Arc<spin::Mutex<CowExt>> = 
        Arc::new(spin::Mutex::new(CowExt::new()));
}

pub fn cow_table() -> spin::MutexGuard<'static, CowExt>{
    COW_TABLE.lock()
}

/*
* @brief:
*   allocate a free physical frame, if no free frame, then swap out one page and reture mapped frame as the free one
* @retval:
*   the physical address for the allocated frame
*/
pub fn alloc_frame() -> Option<usize> {
    // get the real address of the alloc frame
    let ret = FRAME_ALLOCATOR.lock().alloc().map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
    trace!("Allocate frame: {:x?}", ret);
    //do we need : unsafe { ACTIVE_TABLE_SWAP.force_unlock(); } ???
    Some(ret.unwrap_or_else(|| {
        // here we should get the active_table's lock before we get the swap_table since in memroy_set's map function
        // we get pagetable before we get the swap table lock
        // otherwise we may run into dead lock
        let mut temp_table = active_table();
        swap_table().swap_out_any(temp_table.get_data_mut()).ok().expect("fail to swap out page")
    }))
}

pub fn dealloc_frame(target: usize) {
    trace!("Deallocate frame: {:x}", target);
    FRAME_ALLOCATOR.lock().dealloc((target - MEMORY_OFFSET) / PAGE_SIZE);
}

pub struct KernelStack(usize);
const STACK_SIZE: usize = 0x8000;

impl KernelStack {
    pub fn new() -> Self {
        use alloc::alloc::{alloc, Layout};
        let bottom = unsafe{ alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()) } as usize;
        KernelStack(bottom)
    }
    pub fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        use alloc::alloc::{dealloc, Layout};
        unsafe{ dealloc(self.0 as _, Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()); }
    }
}


/*
* @param:
*   addr: the virtual address of the page fault
* @brief:
*   handle page fault
* @retval:
*   Return true to continue, false to halt
*/
pub fn page_fault_handler(addr: usize) -> bool {
    info!("start handling swap in/out page fault");
    //unsafe { ACTIVE_TABLE_SWAP.force_unlock(); }
    
    info!("active page table token in pg fault is {:x?}, virtaddr is {:x?}", ActivePageTable::token(), addr);
    /*
    let mmset_record = memory_set_record();
    let id = mmset_record.iter()
            .position(|x| unsafe{(*(x.clone() as *mut MemorySet)).get_page_table_mut().token() == ActivePageTable::token()});
    /*LAB3 EXERCISE 1: YOUR STUDENT NUMBER
    * handle the frame deallocated
    */
    assert!(!id.is_none());
    match id {
        Some(targetid) => {
            info!("get id from memroy set recorder.");
            let mmset_ptr = mmset_record.get(targetid).expect("fail to get mmset_ptr").clone();
            // get current mmset

            let current_mmset = unsafe{&mut *(mmset_ptr as *mut MemorySet)};
            //check whether the vma is legal
            if current_mmset.find_area(addr).is_none(){
                return false;
            }

            let pt = current_mmset.get_page_table_mut();
            info!("pt got!");
            if swap_table().page_fault_handler(active_table().get_data_mut(), pt as *mut InactivePageTable0, addr, false, || alloc_frame().expect("fail to alloc frame")){
                return true;
            }
        },
        None => {
            info!("get pt from processor()");
            if process().get_memory_set_mut().find_area(addr).is_none(){
                return false;
            }

            let pt = process().get_memory_set_mut().get_page_table_mut();
            info!("pt got");
            let mut temp_table = active_table();
            if swap_table().page_fault_handler(temp_table.get_data_mut(), pt as *mut InactivePageTable0, addr, true, || alloc_frame().expect("fail to alloc frame")){
                return true;
            }
        },
    };
    */
    info!("get pt from process()");
    /*
    if process().get_memory_set_mut().find_area(addr).is_none(){
        return false;
    }

    let pt = process().get_memory_set_mut().get_page_table_mut();
    info!("pt got");
    let mut temp_table = active_table();
    if swap_table().page_fault_handler(temp_table.get_data_mut(), pt as *mut InactivePageTable0, addr, true, || alloc_frame().expect("fail to alloc frame")){
        return true;
    }
    */
    let target_area = process().get_memory_set_mut().find_area(addr);
    match target_area{
        Some(area) => {
            let pt = process().get_memory_set_mut().get_page_table_mut();
            let mut temp_table = active_table();
            if area.page_fault_handler(temp_table.get_data_mut(), pt as *mut InactivePageTable0 as usize, addr) {
                return true;
            }
            //if swap_table().page_fault_handler(temp_table.get_data_mut(), pt as *mut InactivePageTable0, addr, true, || alloc_frame().expect("fail to alloc frame")){
            //    return true;
            //}
        },
        None => {
            return false;
        },
    };
    //////////////////////////////////////////////////////////////////////////////


    // Handle copy on write (not being used now)
    /*
    unsafe { ACTIVE_TABLE.force_unlock(); }
    if active_table().page_fault_handler(addr, || alloc_frame().expect("fail to alloc frame")){
        return true;
    }
    */
    false
}

pub fn init_heap() {
    use consts::KERNEL_HEAP_SIZE;
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe { HEAP_ALLOCATOR.lock().init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE); }
    info!("heap init end");
}

//pub mod test {
//    pub fn cow() {
//        use super::*;
//        use ucore_memory::cow::test::test_with;
//        test_with(&mut active_table());
//    }
//}

/// MemoryHandler for kernel memory
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SimpleMemoryHandler{
    start_addr: VirtAddr,
    phys_start_addr: PhysAddr,
    flags: MemoryAttr,
}

impl MemoryHandler for SimpleMemoryHandler{
    //type Active = ActivePageTable;
    //type Inactvie = InactivePageTable0;
    fn box_clone(&self) -> Box<MemoryHandler>{
        Box::new((*self).clone())
    }

    fn map(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        let target = addr - self.start_addr + self.phys_start_addr;
        self.flags.apply(pt.map(addr, target));
    }

    fn unmap(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        pt.unmap(addr);
    }
    
    fn page_fault_handler(&self, page_table: &mut PageTable, inpt: usize, addr: VirtAddr) -> bool{
        false
    }

    fn map_clone(&mut self, inpt: usize, addr: VirtAddr){
        info!("come into SimpleMemoryHandler map_clone, the addr is {:x?}", addr);
        unsafe{
            let Self {ref start_addr, ref phys_start_addr, ref flags} = self;
            let mut page_table = &mut *(inpt as *mut InactivePageTable0);
            page_table.edit(|pt|{
                let target = addr - *start_addr + *phys_start_addr;
                flags.apply(pt.map(addr, target));
            });

            let data: Vec<u8> = Vec::from(slice::from_raw_parts(addr as *const u8, PAGE_SIZE));
            page_table.with(||{
                let page_mut = slice::from_raw_parts_mut(addr as *mut u8, PAGE_SIZE);
                page_mut.copy_from_slice(data.as_slice());
            });
        }
    }
    
}

impl SimpleMemoryHandler{
    pub fn new(start_addr: VirtAddr, phys_start_addr: PhysAddr, flags: MemoryAttr) -> Self {
        SimpleMemoryHandler{
            start_addr,
            phys_start_addr,
            flags,
        }
    }
}

#[derive(Clone)]
pub struct NormalMemoryHandler{
    flags: MemoryAttr,
}


impl MemoryHandler for NormalMemoryHandler{
    //type Active = ActivePageTable;
    //type Inactvie = InactivePageTable0;
    fn box_clone(&self) -> Box<MemoryHandler>{
        Box::new((*self).clone())
    }

    fn map(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        let target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
        self.flags.apply(pt.map(addr, target));
    }

    fn unmap(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        let target = pt.get_entry(addr).expect("fail to get entry").target();
        InactivePageTable0::dealloc_frame(target);
        pt.unmap(addr);
    }
    
    fn page_fault_handler(&self, page_table: &mut PageTable, inpt: usize, addr: VirtAddr) -> bool {
        false
    }

    fn map_clone(&mut self, inpt: usize, addr: VirtAddr){
        unsafe{
            let Self {ref flags} = self;
            let mut page_table = &mut *(inpt as *mut InactivePageTable0);
            page_table.edit(|pt|{
                let target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
                flags.apply(pt.map(addr, target));
            });
            let data: Vec<u8> = Vec::from(slice::from_raw_parts(addr as *const u8, PAGE_SIZE));
            page_table.with(||{
                let page_mut = slice::from_raw_parts_mut(addr as *mut u8, PAGE_SIZE);
                page_mut.copy_from_slice(data.as_slice());
            });
        }
    }
}

impl NormalMemoryHandler{
    pub fn new(flags: MemoryAttr) -> Self {
        NormalMemoryHandler{
            flags,
        }
    }
}

pub struct SwapMemoryHandler{
    swap_ext: Arc<spin::Mutex<SwapExtType>>,
    flags: MemoryAttr,
    //delay_alloc: bool,
    delay_alloc: Vec<VirtAddr>,
    //page_table: *mut InactivePageTable0,
}

impl MemoryHandler for SwapMemoryHandler{
    //type Active = ActivePageTable;
    //type Inactvie = InactivePageTable0;
    fn box_clone(&self) -> Box<MemoryHandler>{
        Box::new((*self).clone())
    }

    fn map(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        let id = self.delay_alloc.iter().position(|x|*x == addr);
        if id.is_some(){
            info!("delay allocated addr: {:x?}", addr);
            {
                let entry = pt.map(addr,0);
                self.flags.apply(entry);
            }
            let entry = pt.get_entry(addr).expect("fail to get entry");
            entry.set_present(false);
            entry.update();
        }
        else{
            info!("no delay allocated addr: {:x?}", addr);
            let target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
            self.flags.apply(pt.map(addr, target));
            unsafe{self.swap_ext.lock().set_swappable(pt, inpt as *mut InactivePageTable0, addr);}
        }
    }

    fn unmap(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        unsafe{
            self.swap_ext.lock().remove_from_swappable(pt, inpt as *mut InactivePageTable0, addr, || InactivePageTable0::alloc_frame().expect("alloc frame failed"));
        }
        if pt.get_entry(addr).expect("fail to get entry").present(){
            let target = pt.get_entry(addr).expect("fail to get entry").target();
            InactivePageTable0::dealloc_frame(target);
        }
        else{
            // set valid for pt.unmap function
            pt.get_entry(addr).expect("fail to get entry").set_present(true);
        }
        pt.unmap(addr);
    }
    
    fn page_fault_handler(&self, page_table: &mut PageTable, inpt: usize, addr: VirtAddr) -> bool {
        //self.swap_ext.lock().page_fault_handler(page_table, inpt as *mut InactivePageTable0, addr, true, || InactivePageTable0::alloc_frame().expect("alloc frame failed"))
        // handle page delayed allocating
        
        let id = self.delay_alloc.iter().position(|x|*x == addr);
        if id.is_some(){
            info!("try handling delayed frame allocator");
            let need_alloc ={
                let entry = page_table.get_entry(addr).expect("fail to get entry");
                //info!("got entry!");
                !entry.present() && !entry.swapped()
            };
            info!("need_alloc got");
            if need_alloc {
                info!("need_alloc!");
                let frame = InactivePageTable0::alloc_frame().expect("alloc frame failed");
                {
                    let entry = page_table.get_entry(addr).expect("fail to get entry");
                    entry.set_target(frame);
                    //let new_entry = self.page_table.map(addr, frame);
                    entry.set_present(true);
                    entry.update();
                }
                unsafe{self.swap_ext.lock().set_swappable(page_table, inpt as *mut InactivePageTable0, Page::of_addr(addr).start_address())};
                //area.get_flags().apply(new_entry); this instruction may be used when hide attr is used
                info!("allocated successfully");
                return true;
            }
            info!("not need alloc!");
        }
        // handle the swap out page fault            
        // now we didn't attach the cow so the present will be false when swapped(), to enable the cow some changes will be needed
        match page_table.get_entry(addr) {
            // infact the get_entry(addr) should not be None here
            None => return false,
            Some(entry) => if !(entry.swapped() && !entry.present())  { return false; },
        }
        // Allocate a frame, if failed, swap out a page
        let frame = InactivePageTable0::alloc_frame().expect("alloc frame failed");
        self.swap_ext.lock().swap_in(page_table, inpt as *mut InactivePageTable0, addr, frame).ok().unwrap();
        true
        
    }
    
    fn map_clone(&mut self, inpt: usize, addr: VirtAddr){
        info!("Come into SwapMemoryHandler map_clone, the addr is {:x?}", addr);
        let Self {ref swap_ext, ref flags, ref mut delay_alloc} = self;
        let mut allocated = {
            let mut temp_table = active_table();
            let entry = temp_table.get_entry(addr).expect("fail to get entry");
            entry.present() || entry.swapped()
            // infact 0 frame is being allocated, it is dangerous to check allcated by entry.target() == 0
        };
        unsafe{
            let mut page_table = &mut *(inpt as *mut InactivePageTable0);
            //allocated = true; // for test
            if !allocated {
                delay_alloc.push(addr);
                page_table.edit(|pt|{
                    {
                        let entry = pt.map(addr,0);
                        flags.apply(entry);
                    }
                    let entry = pt.get_entry(addr).expect("fail to get entry");
                    entry.set_present(false);
                    entry.update();
                });
            }
            else{
                page_table.edit(|pt|{
                    let target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
                    flags.apply(pt.map(addr, target));
                    swap_ext.lock().set_swappable(pt, inpt as *mut InactivePageTable0, addr);
                });
                let data: Vec<u8> = Vec::from(slice::from_raw_parts(addr as *const u8, PAGE_SIZE));
                page_table.with(||{
                    let page_mut = slice::from_raw_parts_mut(addr as *mut u8, PAGE_SIZE);
                    page_mut.copy_from_slice(data.as_slice());
                });
            }
        }
    }

}


impl SwapMemoryHandler{
    pub fn new(swap_ext: Arc<spin::Mutex<SwapExtType>>, flags: MemoryAttr, delay_alloc: Vec<VirtAddr>) -> Self {
        SwapMemoryHandler{
            swap_ext,
            flags,
            delay_alloc,
        }
    }
}


impl Clone for SwapMemoryHandler{
    fn clone(&self) -> Self{
        // when we fork a new process, all the page need to be map with physical phrame immediately
        SwapMemoryHandler::new(self.swap_ext.clone(), self.flags.clone(), Vec::<VirtAddr>::new())
    }
}


pub struct CowMemoryHandler{
    cow_ext: Arc<spin::Mutex<CowExt>>,
    flags: MemoryAttr,
}


impl MemoryHandler for CowMemoryHandler{
    //type Active = ActivePageTable;
    //type Inactvie = InactivePageTable0;
    fn box_clone(&self) -> Box<MemoryHandler>{
        Box::new((*self).clone())
    }

    fn map(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        //info!("COME INTO COW MAP.");
        let target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
        self.flags.apply(pt.map(addr, target));
        let entry = pt.get_entry(addr).expect("fail to get entry");
        //entry.set_writable(false);
        entry.set_shared(!self.flags.is_readonly());
        entry.update();
        self.cow_ext.lock().map_to_shared(target, !self.flags.is_readonly());
    }

    fn unmap(&self, pt: &mut PageTable, inpt: usize, addr: VirtAddr){
        //info!("COME INTO COW UNMAP. addr is {:x?}", addr);
        let target = pt.get_entry(addr).expect("fail to get entry").target();
        pt.unmap(addr);
        //info!("finish pt.unmap");
        if self.cow_ext.lock().unmap_shared(target, !self.flags.is_readonly()){
            //info!("finish unmap_shared");
            InactivePageTable0::dealloc_frame(target);
        }
        //info!("COME OUT OF COW UNMAP.");
    }
    
    fn page_fault_handler(&self, page_table: &mut PageTable, inpt: usize, addr: VirtAddr) -> bool {
        //info!("COME INTO COW PAGEFAULT HANDLER.");
        if self.flags.is_readonly() {
            return false;
        }
        let target = page_table.get_entry(addr).expect("fail to get entry").target();
        if self.cow_ext.lock().is_one_shared(target){
            let entry = page_table.get_entry(addr).expect("fail to get entry");
            entry.set_writable(true);
            entry.update();
        }
        else{
            unsafe{
                let page_addr = Page::of_addr(addr).start_address();
                let data: Vec<u8> = Vec::from(slice::from_raw_parts(page_addr as *const u8, PAGE_SIZE));
                self.cow_ext.lock().unmap_shared(target, true);
                let new_target = InactivePageTable0::alloc_frame().expect("failed to allocate frame");
                let entry = page_table.get_entry(addr).expect("fail to get entry");
                entry.set_writable(true);
                entry.set_target(new_target);
                entry.update();
                self.cow_ext.lock().map_to_shared(new_target, !self.flags.is_readonly());
                let page_mut = slice::from_raw_parts_mut(page_addr as *mut u8, PAGE_SIZE);
                page_mut.copy_from_slice(data.as_slice());
            }
        }
        true
    }

    fn map_clone(&mut self, inpt: usize, addr: VirtAddr){
        //info!("COME INTO COW MAP CLONE.");
        unsafe{
            let Self {ref mut cow_ext, ref flags} = self;
            let mut page_table = &mut *(inpt as *mut InactivePageTable0);
            let target = {
                let mut temp_table = active_table();
                let entry = temp_table.get_entry(addr).expect("fail to get entry");
                let ret = entry.target();
                entry.set_writable(false);
                entry.update();
                ret
            };
            page_table.edit(|pt|{
                flags.apply(pt.map(addr, target));
                let entry = pt.get_entry(addr).expect("fail to get entry");
                entry.set_writable(false);
                entry.set_shared(!flags.is_readonly());
                entry.update();
                cow_ext.lock().map_to_shared(target, !flags.is_readonly());
            });
        }
    }
}

impl CowMemoryHandler{
    pub fn new(cow_ext: Arc<spin::Mutex<CowExt>>, flags: MemoryAttr) -> Self {
        CowMemoryHandler{
            cow_ext,
            flags,
        }
    }
}


impl Clone for CowMemoryHandler{
    fn clone(&self) -> Self{
        // when we fork a new process, all the page need to be map with physical phrame immediately
        CowMemoryHandler::new(self.cow_ext.clone(), self.flags.clone())
    }
}
