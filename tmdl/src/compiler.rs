use std::io::Write;
use std::{fs, io};

use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::{ast, emit_rust, lex, parse, SyntaxNodeData};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Action {
    EmitTokens,
    EmitSyntaxTree,
    EmitAst,
    EmitRust,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(value_enum, long)]
    pub action: Action,
    pub input: String,
    #[arg(short, long)]
    pub output: String,
    #[arg(short, long)]
    pub dialect: Option<String>,
}

pub fn compiler_main(args: Option<&ArgMatches>) -> Result<(), Box<dyn std::error::Error>> {
    let args = match args {
        Some(args) => Cli::from_arg_matches(args),
        None => Ok(Cli::parse()),
    }?;

    let source = std::fs::read_to_string(args.input)?;

    let mut output: Box<dyn Write> = if args.output == "-" {
        Box::new(io::BufWriter::new(io::stdout()))
    } else {
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .open(&args.output)?;
        Box::new(io::BufWriter::new(file))
    };

    match args.action {
        Action::EmitTokens => {
            let tokens = lex(&source).unwrap();
            writeln!(output, "{:#?}", tokens)?;
        }
        Action::EmitSyntaxTree => {
            let tokens = lex(&source).unwrap();
            let root = parse(&tokens);
            writeln!(output, "{:#?}", root)?;
        }
        Action::EmitAst => {
            let tokens = lex(&source).unwrap();
            let root = parse(&tokens);
            let red_root = SyntaxNodeData::new(root);
            let translation_unit = ast::SourceFile::new(red_root);
            writeln!(output, "{:#?}", translation_unit)?;
        }
        Action::EmitRust => {
            if args.dialect.is_none() {
                let mut cmd = Cli::command();
                cmd.error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--dialect must be specified with --action=emit-rust",
                )
                .exit();
            }
            let tokens = lex(&source).unwrap();
            let root = parse(&tokens);
            let red_root = SyntaxNodeData::new(root);
            let translation_unit = ast::SourceFile::new(red_root);
            emit_rust(
                &mut output,
                &translation_unit.unwrap(),
                &args.dialect.unwrap(),
            )?;
        }
    }

    Ok(())
}
