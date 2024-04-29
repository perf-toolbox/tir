use crate::{builtin, context, Block, Operation};

pub trait IRText {
    fn print(&self, fmt: &mut dyn IRFormatter);
    fn parse(input: &str) -> Operation
    where
        Self: Sized;
}

pub trait IRFormatter {
    fn increase_indent(&mut self);
    fn decrease_indent(&mut self);
    fn get_indent(&self) -> u32;
    fn write_direct(&mut self, data: &str);

    fn write_newline(&mut self, data: &str) {
        self.write_direct(data);
        self.write_direct("\n");
        for _ in 0..self.get_indent() {
            self.write_direct("  ");
        }
    }

    fn start_block(&mut self) {
        self.write_direct("{\n");
        self.increase_indent();
    }

    fn end_block(&mut self) {
        self.decrease_indent();
        self.write_newline("}");
    }
}

pub struct StdoutPrinter {
    indent: u32,
}

impl StdoutPrinter {
    pub fn new() -> Self {
        StdoutPrinter { indent: 0 }
    }
}

impl IRFormatter for StdoutPrinter {
    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    fn get_indent(&self) -> u32 {
        self.indent
    }

    fn write_direct(&mut self, data: &str) {
        print!("{}", data);
    }
}

/// Prints given operation to stdout
pub fn print_op(op: Operation, fmt: &mut dyn IRFormatter) {
    let context = op.borrow().get_context();
    let dialect = op.borrow().get_dialect_id();
    let dialect = context.borrow().get_dialect(dialect).unwrap();

    if dialect.borrow().get_name() != builtin::DIALECT_NAME {
        fmt.write_direct(&dialect.borrow().get_name());
        fmt.write_direct(".");
    }

    fmt.write_direct(op.borrow().get_op_name());
    fmt.write_direct(" ");
    op.borrow().print(fmt);
}
