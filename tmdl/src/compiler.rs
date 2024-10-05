use clap::{ArgMatches, FromArgMatches, Parser, ValueEnum};

use crate::{lex, parse};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Action {
    EmitTokens,
    EmitSyntaxTree,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(value_enum, long)]
    pub action: Action,
    pub input: String,
    #[arg(short, long)]
    pub output: String,
}

pub fn compiler_main(args: Option<&ArgMatches>) -> Result<(), Box<dyn std::error::Error>> {
    let args = match args {
        Some(args) => Cli::from_arg_matches(args),
        None => Ok(Cli::parse()),
    }?;

    let source = std::fs::read_to_string(args.input)?;

    match args.action {
        Action::EmitTokens => {
            let tokens = lex(&source).unwrap();
            println!("{:#?}", tokens);
        }
        Action::EmitSyntaxTree => {
            let tokens = lex(&source).unwrap();
            let root = parse(&tokens);
            println!("{:#?}", root);
        }
    }

    Ok(())
}
