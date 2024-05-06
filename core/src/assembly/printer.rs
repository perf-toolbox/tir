use crate::{builtin, IRFormatter, OpRef};

/// Prints given operation to stdout
pub fn print_op(op: OpRef, fmt: &mut dyn IRFormatter) {
    let context = op.borrow().get_context();
    let dialect = op.borrow().get_dialect_id();
    let dialect = context.get_dialect(dialect).unwrap();

    if dialect.get_name() != builtin::DIALECT_NAME {
        fmt.write_direct(dialect.get_name());
        fmt.write_direct(".");
    }

    fmt.write_direct(op.borrow().get_operation_name());
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
        let module = ModuleOp::builder(&context).build();

        let mut printer = StringPrinter::new();

        print_op(module, &mut printer);

        let result = printer.get();

        let golden = "module {\n}\n";
        assert_eq!(result, golden);
    }
}
