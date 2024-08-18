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
        for region in &mut self.regions {
            let last_address = region.address + region.size;
            if address >= region.address && address <= last_address {
                let start = (address - region.address) as usize;
                let end = start + data.len();
                let dst = &mut region.data[start..end];
                dst.clone_from_slice(&data);
                return Ok(());
            }
        }

        Err(())
    }

    pub fn load(&self, address: u64, size: u8) -> Result<Vec<u8>, ()> {
        for region in &self.regions {
            let last_address = region.address + region.size;
            if address >= region.address && address <= last_address {
                let start = (address - region.address) as usize;
                let end = start + size as usize;
                let dst = &region.data[start..end];
                return Ok(dst.to_vec());
            }
        }

        Err(())
    }

    pub fn dump(&self) -> String {
        let mut mem = String::new();
        for r in &self.regions {
            mem += &format!(
                "start = {}\nsize = {}\n{:?}\n---\n",
                r.address, r.size, &r.data
            );
        }

        mem
    }
}
