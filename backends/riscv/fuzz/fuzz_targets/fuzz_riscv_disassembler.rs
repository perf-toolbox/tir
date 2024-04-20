#![no_main]

use libfuzzer_sys::fuzz_target;
use tir_riscv::disassemble;

fuzz_target!(|data: &[u8]| {
    let _ = disassemble(data);
});
