pub mod target;
mod target_options;
pub use target_options::*;

use tir_core::{Operation, Result};

pub trait BinaryStream {
    fn write(&mut self, data: &[u8]);
}

pub trait BinaryEmittable {
    fn encode(&self, target_opts: &TargetOptions, stream: &mut Box<dyn BinaryStream>)
        -> Result<()>;
    fn try_decode(data: &[u8]) -> Result<Operation>;
}

pub trait AsmPrintable {
    fn print(&self, target_opts: &TargetOptions);
    fn try_parse(instruction: &str) -> Result<Operation>;
}
