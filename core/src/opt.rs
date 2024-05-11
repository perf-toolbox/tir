use std::env;

use crate::{parse_ir, parser::print_parser_diag, Context, StdoutPrinter};

pub fn opt_main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let context = Context::new();

    let ir = std::fs::read_to_string(path).unwrap();

    let module = parse_ir(context.clone(), &ir);

    match module {
        Ok(module) => {
            let mut printer = StdoutPrinter::new();
            module.borrow().print(&mut printer);
        }
        Err(err) => {
            print_parser_diag(context, &err);
            std::process::exit(1);
        }
    }
}
