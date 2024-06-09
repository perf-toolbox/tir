use tir_core::Context;
use tir_tools::tir_main;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::new();

    // TODO: refactor into a separate function available to every downstream crate
    context.add_dialect(tir_riscv::create_dialect());
    context.add_dialect(tir_backend::target::create_dialect());
    context.add_dialect(tir_backend::isema::create_dialect());

    tir_main(context)
}
