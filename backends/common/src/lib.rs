mod diagnostic;
pub mod isema;
mod lexer;
pub mod parser;
pub mod target;
mod target_options;

pub use diagnostic::*;
pub use lexer::*;
use lpl::{combinators::NotTuple, ParseResult};
pub use target_options::*;

use tir_core::{parser::Parsable, Attr, Printable, Result};

use thiserror::Error;

#[derive(Clone, Copy)]
pub enum Register<T: Into<Register<T>> + Printable + Parsable<T> + Copy> {
    Virtual(u64),
    Architecture(T),
}

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

impl<T: Into<Register<T>> + Printable + Parsable<T> + Copy> Printable for Register<T> {
    fn print(&self, fmt: &mut dyn tir_core::IRFormatter) {
        match &self {
            Register::Virtual(virt) => fmt.write_direct(&format!("virt_reg<{}>", virt)),
            Register::Architecture(reg) => reg.print(fmt),
        }
    }
}

impl<T: Into<Register<T>> + Printable + Parsable<T> + Copy> Parsable<Register<T>> for Register<T> {
    fn parse(input: tir_core::IRStrStream) -> ParseResult<tir_core::IRStrStream, Register<T>> {
        let (reg, ni) = T::parse(input)?;
        Ok((Register::Architecture(reg), ni))
    }
}

#[allow(clippy::from_over_into)]
impl<T: Into<Register<T>> + Printable + Parsable<T> + Into<tir_core::Attr> + Copy>
    Into<tir_core::Attr> for Register<T>
{
    fn into(self) -> tir_core::Attr {
        match self {
            Register::Virtual(virt) => Attr::U64(virt),
            Register::Architecture(arch) => arch.into(),
        }
    }
}

impl<T: Into<Register<T>> + Printable + Parsable<T> + Copy> Register<T> {
    pub fn as_arch(&self) -> T {
        match &self {
            Register::Architecture(arch) => *arch,
            _ => unreachable!(),
        }
    }
}

impl<T: Into<Register<T>> + Printable + Parsable<T> + Copy> NotTuple for Register<T> {}
