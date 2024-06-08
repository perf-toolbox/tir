use std::env;

use crate::{parse_ir, parser::print_parser_diag, Context, StdoutPrinter};

pub fn opt_main() {
    let args: Vec<String> = env::args().collect();

    let path: String = if args.len() > 1 {
        String::from(&args[1])
    } else {
        String::from("-")
    };

    let context = Context::new();

    let ir = if path == "-" {
        if let Ok(i) = std::io::read_to_string(std::io::stdin()) {
            i
        } else {
            panic!("Could not stdin")
        }
    } else {
        if let Ok(i) = std::fs::read_to_string(path) {
            i
        } else {
            panic!("Could not read file")
        }
    };

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
