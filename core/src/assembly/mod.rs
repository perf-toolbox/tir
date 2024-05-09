mod formatter;
pub mod parser;
mod printer;

pub use formatter::*;
pub use printer::*;

pub use crate::parser::parse_ir;

use crate::OpRef;

pub trait OpAssembly {
    fn print_assembly(&self, fmt: &mut dyn IRFormatter);
    fn parse_assembly(input: &mut parser::ParseStream<'_>) -> parser::PResult<OpRef>
    where
        Self: Sized;
}
