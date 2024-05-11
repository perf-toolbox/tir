use crate::{BlockRef, IRFormatter, RegionRef};

pub trait Printable {
    fn print(&self, fmt: &mut dyn IRFormatter);
}

/// Prints given values as a comma separated list
pub fn print_comma_separated(fmt: &mut dyn IRFormatter, tokens: &[String]) {
    // FIXME: come up with zero allocation way
    let tokens = tokens.join(", ");
    fmt.write_direct(&tokens);
}

pub fn print_block(fmt: &mut dyn IRFormatter, block: &BlockRef) {
    fmt.indent();
    fmt.write_direct(&format!("^{}:\n", block.get_name()));

    for op in block.iter() {
        op.borrow().print(fmt);
    }
}

/// Prints region with all its blocks and their names and operations.
/// Automatically adds opening and closing brackets.
pub fn print_region(fmt: &mut dyn IRFormatter, region: &RegionRef) {
    fmt.start_region();
    for block in region.iter() {
        print_block(fmt, &block);
    }
    fmt.end_region();
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
