use std::cell::RefCell;
use std::rc::Rc;

struct Region {
    address: u64,
    size: u64,
    data: Vec<u8>,
}

impl Region {
    pub fn new(address: u64, size: u64) -> Self {
        let mut data = Vec::new();
        data.resize(size as usize, 0);
        Self {
            address,
            size,
            data,
        }
    }
}

pub struct MemoryMap {
    regions: Vec<Region>,
}

impl MemoryMap {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { regions: vec![] }))
    }

    pub fn add_region(&mut self, address: u64, size: u64) {
        self.regions.push(Region::new(address, size))
    }

    pub fn store(&mut self, address: u64, data: &[u8]) -> Result<(), ()> {
        for region in &self.regions {
            let last_address = region.address + region.size;
            if address >= region.address && address <= last_address {
                let offset = (address - region.address) as usize;
                let mut dst = &region.data[offset..data.len()];
                dst.clone_from(&data);
            }
        }

        Err(())
    }
}
