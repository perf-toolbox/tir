mod elf64;

pub trait BinaryEmitter {
    fn encode(&self);
}

pub use elf64::*;
