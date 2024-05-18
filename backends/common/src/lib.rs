pub mod target;
mod target_options;
pub mod parser;
mod tokenizer;

pub use target_options::*;
use thiserror::Error;

use tir_core::Result;

pub trait BinaryStream {
    fn write(&mut self, data: &[u8]);
}

pub trait BinaryEmittable {
    fn encode(&self, target_opts: &TargetOptions, stream: &mut Box<dyn BinaryStream>)
        -> Result<()>;
}

pub trait AsmPrintable {
    fn print(&self, target_opts: &TargetOptions)
    where
        Self: Sized;
}

#[derive(Error, Debug)]
pub enum DisassemblerError {
    #[error("unexpected end of stream, need `{0}` more bytes, only `{1}` bytes left")]
    UnexpectedEndOfStream(usize, usize),
    #[error("unknown disassembler error")]
    Unknown,
}
