use crate::{builtin, IRFormatter, Operation};

/// Prints given operation to stdout
pub fn print_op(op: Operation, fmt: &mut dyn IRFormatter) {
    let context = op.borrow().get_context();
    let dialect = op.borrow().get_dialect_id();
    let dialect = context.borrow().get_dialect(dialect).unwrap();

    if dialect.borrow().get_name() != builtin::DIALECT_NAME {
        fmt.write_direct(dialect.borrow().get_name());
        fmt.write_direct(".");
    }

    fmt.write_direct(op.borrow().get_op_name());
    fmt.write_direct(" ");
    op.borrow().print(fmt);
}

#[cfg(test)]
mod tests {
    use super::print_op;
    use crate::builtin::ModuleOp;
    use crate::Context;
    use crate::StringPrinter;

    #[test]
    fn test_module_print() {
        let context = Context::new();
        let module = ModuleOp::builder(context).build();

        let mut printer = StringPrinter::new();

        print_op(module, &mut printer);

        let result = printer.get();

        let golden = "module {\n}\n";
        assert_eq!(result, golden);
    }
}
