use clap::{ArgMatches, FromArgMatches, Parser};
use std::cell::RefCell;
use std::rc::Rc;
use tir_core::{builtin::ModuleOp, Context, ContextRef, OpRef, PassManager};

mod memory;
mod options;
mod regfile;
mod simulator;

pub use memory::*;
pub use options::*;
pub use regfile::*;
pub use simulator::*;

pub fn sim_main(
    context: Option<ContextRef>,
    args: Option<&ArgMatches>,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = match args {
        Some(args) => Cli::from_arg_matches(args),
        None => Ok(Cli::parse()),
    }?;

    let context = match context {
        Some(context) => context,
        None => {
            let context = Context::new();
            // TODO: refactor into a separate function available to every downstream crate
            context.add_dialect(tir_riscv::create_dialect());
            context.add_dialect(tir_backend::target::create_dialect());
            context.add_dialect(tir_backend::isema::create_dialect());
            context
        }
    };

    let config = std::fs::read_to_string(args.experiment)?;
    let config: Config = serde_yml::from_str(&config)?;

    let reg_file: Rc<RefCell<dyn RegFile>> = RISCVRegFile::new();

    for (name, value) in &config.register_state {
        reg_file.borrow_mut().write_register(&name, &value.into());
    }

    let asm = std::fs::read_to_string(args.input)?;
    let asm: OpRef = tir_riscv::parse_asm(&context, &asm).unwrap();

    let pm = PassManager::new_from_list(&["convert-asm-to-isema"])?;
    if let Err(e) = pm.run(&asm) {
        eprintln!("{:?}", e);
        std::process::exit(0);
    }

    let asm = tir_core::utils::op_cast::<ModuleOp>(asm).unwrap();

    let simulator = Simulator::new(asm);
    simulator.run(&reg_file);

    println!("{}", reg_file.borrow().dump());

    Ok(())
}
