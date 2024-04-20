#![no_main]

use libfuzzer_sys::fuzz_target;
use tir_core::Context;
use tir_riscv::disassemble;

fuzz_target!(|data: &[u8]| {
    let context = Context::new();
    context
        .borrow_mut()
        .add_dialect(tir_riscv::create_dialect());
    let _ = disassemble(&context, data);
});
