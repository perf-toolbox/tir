mod formatter;
pub mod parser;
mod printer;

use crate::{Attr, OpRef};
use std::collections::HashMap;

pub use formatter::*;
pub use parser::parse_ir;
pub use printer::*;

pub trait OpAssembly {
    fn print_assembly(&self, fmt: &mut dyn IRFormatter);
    fn parse_assembly(input: &mut parser::ParseStream<'_>) -> parser::PResult<OpRef>
    where
        Self: Sized;
}

pub trait TyAssembly {
    fn print_assembly(attrs: &HashMap<String, Attr>, fmt: &mut dyn IRFormatter)
    where
        Self: Sized;
    fn parse_assembly(
        input: &mut parser::ParseStream<'_>,
    ) -> parser::PResult<HashMap<String, Attr>>
    where
        Self: Sized;
}
