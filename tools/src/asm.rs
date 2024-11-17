use clap::{ArgMatches, FromArgMatches, Parser};
use tir_core::{ContextRef, Printable, StdoutPrinter};

#[derive(Debug, Parser)]
#[command(name = "asm")]
pub struct Cli {
    #[arg(default_value = "-")]
    input: String,
}

pub fn main(
    context: ContextRef,
    args: Option<&ArgMatches>,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = match args {
        Some(args) => Cli::from_arg_matches(args),
        None => Ok(Cli::parse()),
    }?;

    let ir = if args.input == "-" {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(args.input)?
    };

    let module = tir_riscv::parse_asm(&context, &ir);

    match module {
        Ok(module) => {
            let mut printer = StdoutPrinter::new();
            module.borrow().print(&mut printer);
        }
        Err(err) => {
            // FIXME figure out how to tie syntax errors with tokens
            println!("{:?}", err);
            panic!("ASM syntax error");
        }
    }

    Ok(())
}
