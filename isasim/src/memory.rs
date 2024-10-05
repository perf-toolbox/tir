use std::cell::RefCell;
use std::rc::Rc;

use crate::SimErr;

struct Region {
    address: u64,
    size: u64,
    data: Vec<u8>,
}

impl Region {
    pub fn new(address: u64, size: u64) -> Self {
        let data = vec![0; size as usize];
        Self {
            address,
            size,
            data,
        }
    }
}

pub struct MemoryMap {
    regions: Vec<Region>,
    page_size: u64,
    fault_page_address: Option<u64>,
    unaligned_access: bool,
}

impl MemoryMap {
    pub fn new(page_size: u64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            regions: vec![],
            page_size,
            fault_page_address: None,
            unaligned_access: true,
        }))
    }

    pub fn set_map_faults_to_address(&mut self, address: u64) {
        self.fault_page_address = Some(address);
    }

    pub fn prohibit_unaligned_access(&mut self) {
        self.unaligned_access = false;
    }

    pub fn add_region(&mut self, address: u64, size: u64) {
        self.regions.push(Region::new(address, size))
    }

    fn find_region_id(&self, address: u64) -> Result<(usize, u64), SimErr> {
        let find = |regions: &Vec<Region>, address| {
            for (id, region) in regions.iter().enumerate() {
                let last_address = region.address + region.size;

                if address >= region.address && address <= last_address {
                    return Ok(id);
                }
            }

            Err(SimErr::MemoryAccess(address))
        };

        if let Ok(id) = find(&self.regions, address) {
            return Ok((id, address));
        }

        if let Some(fault_addr) = self.fault_page_address {
            let offset = address % self.page_size;
            let new_addr = fault_addr + offset;
            return find(&self.regions, new_addr).map(|id| (id, new_addr));
        }

        Err(SimErr::MemoryAccess(address))
    }

    pub fn store(&mut self, address: u64, data: &[u8]) -> Result<(), SimErr> {
        if address % data.len() as u64 != 0 && !self.unaligned_access {
            return Err(SimErr::UnalignedAccess(address, data.len()));
        }

        let (region_id, address) = self.find_region_id(address)?;
        let region = &mut self.regions[region_id];
        let start = (address - region.address) as usize;
        let end = start + data.len();
        let dst = &mut region.data[start..end];
        dst.clone_from_slice(data);
        Ok(())
    }

    pub fn load(&self, address: u64, size: u8) -> Result<Vec<u8>, SimErr> {
        if address % size as u64 != 0 && !self.unaligned_access {
            return Err(SimErr::UnalignedAccess(address, size as usize));
        }

        let (region_id, address) = self.find_region_id(address)?;
        let region = &self.regions[region_id];
        let start = (address - region.address) as usize;
        let end = start + size as usize;
        let dst = &region.data[start..end];
        Ok(dst.to_vec())
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
