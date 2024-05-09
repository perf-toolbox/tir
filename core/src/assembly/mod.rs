mod formatter;
mod parser;
mod printer;

pub use formatter::*;
pub use parser::*;
pub use printer::*;

use crate::ContextRef;


pub trait Assembly<T> {
    fn print_ir(&self, fmt: &mut dyn IRFormatter);
    fn parse_ir(context: ContextRef, input: &mut &str) -> std::result::Result<T, ()>
    where
        Self: Sized;
}
