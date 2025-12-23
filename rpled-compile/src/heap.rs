

struct Heap {
    allocations: Vec<u8>,
}

impl Heap {

    fn new() -> Self {
        Heap {
            allocations: Vec::new(),
        }
    }

    fn allocate(&mut self, size: usize) -> usize {
        let addr = self.allocations.len();
        self.allocations.resize(addr + size, 0);
        addr
    }

    
}