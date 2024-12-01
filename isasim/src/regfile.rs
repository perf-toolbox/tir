use std::{cell::RefCell, rc::Rc};

const MAX_REG_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct Value {
    data: [u8; MAX_REG_SIZE],
}

macro_rules! value_from_impl {
    ($ty:ty, $vty:ty) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                let mut data: [u8; MAX_REG_SIZE] = [0; MAX_REG_SIZE];
                let value_bytes = value.to_le_bytes();

                #[allow(clippy::manual_memcpy)]
                for i in 0..std::mem::size_of::<$vty>() {
                    data[i] = value_bytes[i];
                }

                Self { data }
            }
        }
    };
}

macro_rules! value_from {
    ($ty:ty) => {
        value_from_impl!($ty, $ty);
        value_from_impl!(&$ty, $ty);
    };
}

impl TryFrom<Vec<u8>> for Value {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() > MAX_REG_SIZE {
            return Err(());
        }

        let mut data: [u8; MAX_REG_SIZE] = [0; MAX_REG_SIZE];

        #[allow(clippy::manual_memcpy)]
        for i in 0..value.len() {
            data[i] = value[i];
        }

        Ok(Self { data })
    }
}

impl Value {
    pub fn get_lower32(&self) -> u32 {
        u32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    pub fn get_lower64(&self) -> u64 {
        u64::from_le_bytes(self.data[0..8].try_into().unwrap())
    }

    #[allow(clippy::result_unit_err)]
    pub fn raw_bytes(&self, width: usize) -> Result<Vec<u8>, ()> {
        if width > MAX_REG_SIZE {
            return Err(());
        }

        Ok(self.data[0..width].to_vec())
    }

    pub fn dump(&self) -> String {
        format!("{:?}", &self.data)
    }
}

impl Default for Value {
    fn default() -> Self {
        let data = [0; MAX_REG_SIZE];
        Self { data }
    }
}

value_from!(u64);
value_from!(u32);
value_from!(u16);
value_from!(u8);
value_from!(i64);
value_from!(i32);
value_from!(i16);
value_from!(i8);

pub trait RegFile {
    fn read_register(&self, reg_name: &str) -> Value;
    fn write_register(&mut self, reg_name: &str, value: &Value);
    fn base_width(&self) -> u8;
    fn dump(&self) -> String;
}

#[derive(Debug)]
pub struct RISCVRegFile {
    registers: Vec<Value>,
    base_width: u8,
}

impl RISCVRegFile {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut registers = vec![];
        registers.resize(32, Value::default());

        Rc::new(RefCell::new(Self {
            registers,
            base_width: 8,
        }))
    }
}

impl RegFile for RISCVRegFile {
    fn base_width(&self) -> u8 {
        self.base_width
    }

    fn read_register(&self, reg_name: &str) -> Value {
        let reg = tir_riscv::parse_gpr(reg_name).unwrap();
        self.registers[reg.get_reg_num()].clone()
    }

    fn write_register(&mut self, reg_name: &str, value: &Value) {
        let reg = tir_riscv::parse_gpr(reg_name).unwrap();

        // hardwired zero
        if let tir_riscv::GPR::X0 = reg {
            return;
        }

        self.registers[reg.get_reg_num()] = value.clone();
    }

    fn dump(&self) -> String {
        let mut strings = vec![];
        strings.push("{".to_string());

        for id in 0..self.registers.len() {
            let reg: tir_riscv::GPR = TryFrom::try_from(id).expect("A valid register");
            strings.push(format!(
                "    \"{}\": {},",
                reg.get_names()[0],
                self.registers[id].get_lower64()
            ));
        }

        strings.push("}".to_string());

        strings.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{RISCVRegFile, RegFile};

    #[test]
    fn riscv_regfile() {
        let reg_file: Rc<RefCell<dyn RegFile>> = RISCVRegFile::new();

        let value = 42;
        reg_file.borrow_mut().write_register("x1", &value.into());
        let other_value = reg_file.borrow().read_register("x1").get_lower32();

        assert_eq!(value, other_value);
    }
}
