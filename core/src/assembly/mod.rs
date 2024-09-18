mod formatter;
mod ir_stream;
pub mod parser;
mod printer;

use crate::{Attr, OpRef};
use std::collections::HashMap;

pub use formatter::*;
use lpl::BoxedParser;
pub use parser::parse_ir;
pub use printer::*;

pub use self::ir_stream::IRStrStream;

pub trait OpAssembly {
    fn print_assembly(&self, fmt: &mut dyn IRFormatter);
    fn parse_assembly<'a>() -> BoxedParser<'a, IRStrStream<'a>, OpRef>
    where
        Self: Sized;
}

pub trait TyAssembly {
    fn print_assembly(attrs: &HashMap<String, Attr>, fmt: &mut dyn IRFormatter)
    where
        Self: Sized;
    fn parse_assembly<'a>() -> BoxedParser<'a, IRStrStream<'a>, HashMap<String, Attr>>
    where
        Self: Sized;
}
