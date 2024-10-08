use clap::{ArgMatches, FromArgMatches, Parser};
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;
use tir_core::Printable;
use tir_core::{builtin::ModuleOp, Context, ContextRef, OpRef, PassManager, StdoutPrinter};

mod memory;
mod options;
mod regfile;
mod simulator;

pub use memory::*;
pub use options::*;
pub use regfile::*;
pub use simulator::*;

#[derive(Error, Debug, Clone)]
pub enum SimErr {
    #[error("invalid memory access to address {0}")]
    MemoryAccess(u64),
    #[error("unaligned access: address `{0}`, data size `{1}`")]
    UnalignedAccess(u64, usize),
    #[error("unknown error")]
    Unknown,
}

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
    let mem = MemoryMap::new(config.page_size);
    if let Some(a) = config.map_faults_to_address {
        mem.borrow_mut().set_map_faults_to_address(a);
    }

    for (name, value) in &config.register_state {
        reg_file.borrow_mut().write_register(name, &value.into());
    }
    if let Some(memory) = config.memory {
        for entry in &memory {
            let bytes = entry.value.to_le_bytes();
            mem.borrow_mut()
                .add_region(entry.address, entry.region_size);
            for i in 0..(entry.region_size / entry.value_size as u64) {
                mem.borrow_mut()
                    .store(
                        entry.address + i * entry.value_size as u64,
                        &bytes[0..entry.value_size as usize],
                    )
                    .expect("err handling");
            }
        }
    }

    if args.dump_memory_before {
        println!("{}", mem.borrow().dump());
    }

    let asm = std::fs::read_to_string(args.input)?;
    let asm: OpRef = tir_riscv::parse_asm(&context, &asm).unwrap();

    let pm = PassManager::new_from_list(&["convert-asm-to-isema"])?;
    if let Err(e) = pm.run(&asm) {
        eprintln!("{:?}", e);
        std::process::exit(0);
    }

    let asm = tir_core::utils::op_cast::<ModuleOp>(asm).unwrap();
    let mut printer = StdoutPrinter::new();
    asm.borrow().print(&mut printer);

    let simulator = Simulator::new(asm);

    let max_repeat = config.repeat.unwrap_or(1);

    for _ in 0..max_repeat {
        simulator.run(&reg_file, &mem)?;
    }

    println!("{}", reg_file.borrow().dump());

    if args.dump_memory_after {
        println!("{}", mem.borrow().dump());
    }

    Ok(())
}
