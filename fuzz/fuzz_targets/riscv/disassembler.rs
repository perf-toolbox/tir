#![no_main]

use libfuzzer_sys::fuzz_target;
use tir_core::{builtin::ModuleOp, Context, OpBuilder};
use tir_riscv::disassemble;

fuzz_target!(|data: &[u8]| {
    let context = Context::new();
    context
        .add_dialect(tir_riscv::create_dialect());
    
    let module = ModuleOp::builder(&context).build();
    
    let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
    
    let _ = disassemble(&context, builder, data);
});
