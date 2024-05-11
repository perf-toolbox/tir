use crate::IRFormatter;

pub trait Printable {
    fn print(&self, fmt: &mut dyn IRFormatter);
}

pub fn print_comma_separated(fmt: &mut dyn IRFormatter, tokens: &[&str]) {
    // FIXME: come up with zero allocation way
    let tokens = tokens.join(", ");
    fmt.write_direct(&tokens);
}

#[cfg(test)]
mod tests {
    use crate::builtin::ModuleOp;
    use crate::Context;
    use crate::Printable;
    use crate::StringPrinter;

    #[test]
    fn test_module_print() {
        let context = Context::new();
        let module = ModuleOp::builder(&context).build();

        let mut printer = StringPrinter::new();

        module.borrow().print(&mut printer);

        let result = printer.get();

        let golden = "module {\n}\n";
        assert_eq!(result, golden);
    }
}
