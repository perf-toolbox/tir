#![no_main]

use libfuzzer_sys::fuzz_target;
use tir_core::{Context, OpBuilder, builtin::ModuleOp};
use tir_riscv::disassemble;

fuzz_target!(|data: &[u8]| {
    let context = Context::new();
    context
        .borrow_mut()
        .add_dialect(tir_riscv::create_dialect());

    let module = ModuleOp::new(context.clone());

    let builder = OpBuilder::new(context.clone(), module.get_body());

    let _ = disassemble(&context, builder, data);
});
