//! Implememnt the swap manager with the FIFO page replacement algorithm

use alloc::collections::VecDeque;
use super::*;


#[derive(Default)]
pub struct FifoSwapManager  {
    deque: VecDeque<Frame>,
}

impl SwapManager for FifoSwapManager {
    fn tick(&mut self) {}

    fn push(&mut self, frame: Frame) {
        info!("SwapManager push token: {:x?} vaddr: {:x?}", frame.get_token(), frame.get_virtaddr());
        self.deque.push_back(frame);
    }

    fn remove(&mut self, token: usize, addr: VirtAddr) {
        info!("SwapManager remove token: {:x?} vaddr: {:x?}", token, addr);
        let id = self.deque.iter()
            .position(|ref x| x.get_virtaddr() == addr && x.get_token() == token)
            .expect("address not found");
        self.deque.remove(id);
        //info!("SwapManager remove token finished: {:x?} vaddr: {:x?}", token, addr);
        
    }

    fn pop<S>(&mut self, _: &mut PageTable, _: &mut S) -> Option<Frame>
        where S: Swapper
    {
        self.deque.pop_front()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    //use swap::test::*;

    #[test]
    fn test() {
        let mut manager = FifoSwapManager::default();
        let pt1 = 0x0;
        let pt1_token = 0x0;
        let pt2 = 0x1;
        let pt2_token = 0x1000;
        manager.push(Frame::new(pt1, 0x0, pt1_token));
        manager.push(Frame::new(pt1, 0x1000, pt1_token));
        manager.push(Frame::new(pt1, 0x2000, pt1_token));
        manager.push(Frame::new(pt2, 0x1000, pt2_token));
        manager.push(Frame::new(pt2, 0x0, pt2_token));
        manager.push(Frame::new(pt1, 0x3000, pt1_token));
        
        manager.remove(pt1_token, 0x2000);
        manager.remove(pt2_token, 0x0);
        use super::mock_swapper::MockSwapper;
        use super::paging::mock_page_table::MockPageTable;
        let mut swapper = MockSwapper::default();
        let mut pagetable = MockPageTable::new();

        assert_eq!(Frame::new(pt1, 0x0, pt1_token), manager.pop(&mut pagetable, &mut swapper).expect("pop failed"));
        assert_eq!(Frame::new(pt1, 0x1000, pt1_token), manager.pop(&mut pagetable, &mut swapper).expect("pop failed"));
        assert_eq!(Frame::new(pt2, 0x1000, pt2_token), manager.pop(&mut pagetable, &mut swapper).expect("pop failed"));
        assert_eq!(Frame::new(pt1, 0x3000, pt1_token), manager.pop(&mut pagetable, &mut swapper).expect("pop failed"));

    }
}
