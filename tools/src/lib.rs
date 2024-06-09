use clap::{Command, CommandFactory};
use tir_core::ContextRef;

pub mod r#asm;
pub mod opt;

fn get_commands() -> [Command; 2] {
    [opt::Cli::command(), r#asm::Cli::command()]
}

pub fn tir_main(ctx: ContextRef) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("tir")
        .arg_required_else_help(true)
        .subcommand_value_name("TOOL")
        .subcommand_help_heading("Tools")
        .subcommands(get_commands());

    let matches = cmd.get_matches();
    let subcommand = matches.subcommand();

    match subcommand {
        Some(("opt", m)) => opt::main(ctx, Some(m)),
        Some(("asm", m)) => r#asm::main(ctx, Some(m)),
        _ => unreachable!("unhandled subcommand"),
    }
}
