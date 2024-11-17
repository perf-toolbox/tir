mod diagnostic;
mod formatter;
mod ir_stream;
pub mod parser;
mod printer;

use crate::{Attr, OpRef};
use std::collections::HashMap;

pub use diagnostic::*;
pub use formatter::*;
use lpl::ParseResult;
pub use parser::parse_ir;
pub use printer::*;

pub use self::ir_stream::IRStrStream;

pub trait OpAssembly {
    fn print_assembly(&self, fmt: &mut dyn IRFormatter);
    fn parse_assembly(input: IRStrStream<'_>) -> ParseResult<IRStrStream<'_>, OpRef>
    where
        Self: Sized;
}

pub trait TyAssembly {
    fn print_assembly(attrs: &HashMap<String, Attr>, fmt: &mut dyn IRFormatter)
    where
        Self: Sized;
    fn parse_assembly(
        input: IRStrStream<'_>,
    ) -> ParseResult<IRStrStream<'_>, HashMap<String, Attr>>
    where
        Self: Sized;
}
