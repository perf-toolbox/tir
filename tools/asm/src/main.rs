use std::env;

use tir_core::{Context, Printable, StdoutPrinter};

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let input = if let Ok(i) = std::fs::read_to_string(path) {
        i
    } else {
        panic!("Could not read file")
    };

    let context = Context::new();
    context.add_dialect(tir_riscv::create_dialect());
    context.add_dialect(tir_backend::target::create_dialect());

    let module = tir_riscv::parse_asm(&context, &input);

    match module {
        Ok(module) => {
            let mut printer = StdoutPrinter::new();
            module.borrow().print(&mut printer);
        }
        Err(_err) => {
            // FIXME figure out how to tie syntax errors with tokens
            // eprintln!("{}", err);
            eprintln!("ASM syntax error");
            std::process::exit(1);
        }
    }
}
