pub mod target;
mod target_options;
pub use target_options::*;

use tir_core::{Operation, Result};

pub trait BinaryStream {
    fn write(&mut self, data: &[u8])
    where
        Self: Sized;
}

pub trait BinaryEmittable {
    fn encode(&self, target_opts: &TargetOptions, stream: &mut Box<dyn BinaryStream>) -> Result<()>
    where
        Self: Sized;
    fn try_decode(data: &[u8]) -> Result<Operation>
    where
        Self: Sized;
}

pub trait AsmPrintable {
    fn print(&self, target_opts: &TargetOptions)
    where
        Self: Sized;
    fn try_parse(instruction: &str) -> Result<Operation>
    where
        Self: Sized;
}
