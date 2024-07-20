use std::{cell::RefCell, rc::Rc};

pub struct Value {
    value: [u8; 256],
}

impl From<&u64> for Value {
    fn from(value: &u64) -> Self {
        todo!()
    }
}

pub trait RegFile {
    fn read_register(&self, reg_name: &str) -> Value;
    fn write_register(&self, reg_name: &str, value: &Value);
}

pub struct RISCVRegFile {
    registers: Vec<u64>,
}

impl RISCVRegFile {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut registers = vec![];
        registers.resize(32, 0);

        Rc::new(RefCell::new(Self { registers }))
    }
}

impl RegFile for RISCVRegFile {
    fn read_register(&self, reg_name: &str) -> Value {
        todo!()
    }

    fn write_register(&self, reg_name: &str, value: &Value) {
        todo!()
    }
}
