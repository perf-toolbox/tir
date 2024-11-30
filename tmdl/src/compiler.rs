use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::{ast, emit_rust, lex, parse, SyntaxNodeData};

pub struct Compiler {
    action: Action,
    inputs: Vec<String>,
    output: OutputKind,
    dialect: Option<String>,
}

pub struct CompilerBuilder {
    action: Option<Action>,
    inputs: Vec<String>,
    output: Option<OutputKind>,
    dialect: Option<String>,
}

#[derive(Clone, Debug)]
pub enum OutputKind {
    File(String),
    Batch(String),
    Stdout,
}

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

impl Compiler {
    pub fn builder() -> CompilerBuilder {
        CompilerBuilder {
            action: None,
            inputs: vec![],
            output: None,
            dialect: None,
        }
    }

    pub fn compile(&self) -> Result<(), Box<dyn std::error::Error>> {
        for input in &self.inputs {
            let mut output: Box<dyn Write> = match &self.output {
                OutputKind::Stdout => Box::new(io::BufWriter::new(io::stdout())),
                OutputKind::File(path) => {
                    let file = fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .open(path)?;
                    Box::new(io::BufWriter::new(file))
                }
                OutputKind::Batch(out_dir) => {
                    let mut path = PathBuf::from(out_dir);
                    path.push(input.replace(".tmdl", ".rs"));

                    fs::create_dir_all(path.parent().as_ref().unwrap())?;

                    let file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .read(true)
                        .open(&path)?;
                    Box::new(io::BufWriter::new(file))
                }
            };

            let source = std::fs::read_to_string(input)?;

            match &self.action {
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
                    if self.dialect.is_none() {
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
                        self.dialect.as_ref().unwrap(),
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl CompilerBuilder {
    pub fn action(self, action: Action) -> Self {
        Self {
            action: Some(action),
            inputs: self.inputs,
            output: self.output,
            dialect: self.dialect,
        }
    }

    pub fn add_input(self, path: &str) -> Self {
        let mut inputs = self.inputs;
        inputs.push(path.to_string());

        Self {
            action: self.action,
            inputs,
            output: self.output,
            dialect: self.dialect,
        }
    }

    pub fn output(self, output: OutputKind) -> Self {
        Self {
            action: self.action,
            inputs: self.inputs,
            output: Some(output),
            dialect: self.dialect,
        }
    }

    pub fn dialect(self, dialect: Option<String>) -> Self {
        Self {
            action: self.action,
            inputs: self.inputs,
            output: self.output,
            dialect,
        }
    }

    pub fn build(self) -> Compiler {
        Compiler {
            action: self.action.unwrap(),
            inputs: self.inputs,
            output: self.output.unwrap(),
            dialect: self.dialect,
        }
    }
}

pub fn compiler_main(args: Option<&ArgMatches>) -> Result<(), Box<dyn std::error::Error>> {
    let args = match args {
        Some(args) => Cli::from_arg_matches(args),
        None => Ok(Cli::parse()),
    }?;

    let output = match args.output.as_str() {
        "-" => OutputKind::Stdout,
        _ => OutputKind::File(args.output.clone()),
    };
    let compiler = Compiler::builder()
        .action(args.action)
        .add_input(&args.input)
        .dialect(args.dialect.clone())
        .output(output)
        .build();

    compiler.compile()
}
