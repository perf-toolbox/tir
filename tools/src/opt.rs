use clap::{ArgMatches, FromArgMatches, Parser};
use tir_core::{parse_ir, parser::print_parser_diag, ContextRef, StdoutPrinter};

#[derive(Debug, Parser)]
#[command(name = "opt")]
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

    let module = parse_ir(context.clone(), &ir);

    match module {
        Ok(module) => {
            let mut printer = StdoutPrinter::new();
            module.borrow().print(&mut printer);
        }
        Err(err) => {
            print_parser_diag(context, &err);
            // FIXME(alexbatashev): return an error instead of exit
            // winnow errors do not implement std::error::Error
            std::process::exit(1);
        }
    }

    Ok(())
}
