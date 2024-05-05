mod formatter;
mod parser;
mod printer;

pub use formatter::*;
pub use parser::*;
pub use printer::*;

use crate::{ContextRef, OpRef};

pub trait Assembly {
    fn print(&self, fmt: &mut dyn IRFormatter);
    fn parse(context: ContextRef, input: &mut &str) -> std::result::Result<OpRef, ()>
    where
        Self: Sized;
}
