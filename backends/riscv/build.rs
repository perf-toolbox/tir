use std::env;
use std::path::PathBuf;

use tmdl::{Action, Compiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_current_dir(env::var(""))
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let compiler = Compiler::builder()
        .action(Action::EmitRust)
        .add_input("defs/registers.tmdl")
        .output(tmdl::OutputKind::Batch(
            out_dir.to_str().unwrap().to_string(),
        ))
        .dialect(Some("riscv".to_string()))
        .build();
    // let generator = Generator::new("tmdl.ungram", syntax_kind.as_path().to_str().unwrap());

    // generator.produce()
    //

    compiler.compile()
}
