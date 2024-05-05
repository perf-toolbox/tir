#![no_main]

use std::str::from_utf8;

use libfuzzer_sys::fuzz_target;
// use tir_core::{parse_ir, Context};

fuzz_target!(|data: &[u8]| {
    // let context = Context::new();
    //
    // if let Ok(ir) = from_utf8(data) {
    //     let _ = parse_ir(context, ir);
    // }
});
