mod diagnostic;
pub mod isema;
mod lexer;
pub mod parser;
pub mod target;
mod target_options;

pub use diagnostic::*;
pub use lexer::*;
use lpl::ParseResult;
pub use target_options::*;

use tir_core::Result;

use thiserror::Error;

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

pub trait ISAParser {
    fn parse(stream: TokenStream) -> ParseResult<TokenStream, ()>;
}

#[derive(Error, Debug)]
pub enum DisassemblerError {
    #[error("unexpected end of stream, need `{0}` more bytes, only `{1}` bytes left")]
    UnexpectedEndOfStream(usize, usize),
    #[error("unknown disassembler error")]
    Unknown,
}
