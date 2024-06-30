//! Instruction semantics modeling dialect
//!
//! The `isema` dialect provides a set of low-level utilities to model the behavior of particular
//! machine instructions. This can be used for a number of applications:
//! 1. Verify low-level optimizations
//! 2. Emulate the execution of a piece of assembly code
//! 3. Search for new optimization patterns
//!
//! Note, however, that at the moment the dialect only captures the *semantics* of an instruction, and does not
//! advertise any architecture-level details, such as register file. Neither it is targeted at
//! modeling microarchitecture details of a particular platform.
//!
//! The general idea of a dialect is that we can model any program as a set of simple operations
//! that do one of the following things:
//! 1. Update registers with immediate values
//! 2. Perform simple arithmetic operations on registers or immediates
//! 3. Load from and store to memory
//!
//! That means, that the semantics of `addi x6, x7, 42` and `vfadd.s v0, v1, v2` are the same and
//! will be represented by `isema.add` operation, even though nothing is similar about these two
//! instructions from architecture point of view: data type, vector width, use of an immediate.

use tir_core::Dialect;
use tir_core::OpAssembly;

mod conversion;
mod ops;
pub use conversion::*;
pub use ops::*;

use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(isema);
populate_dialect_ops!(
    AddOp,
    SubOp,
    AndOp,
    OrOp,
    XorOp,
    SllOp,
    SrlOp,
    SraOp,
    CompInstrOp,
    CompInstrEndOp
);
populate_dialect_types!();

#[macro_export]
macro_rules! def{
    ($op_name:ident => $isema:ty{$($to:ident = $from:ident $(,)?)*}) => {
        #[tir_macros::op_implements]
        impl WithISema for $op_name {
            fn convert(&self, builder: &tir_core::OpBuilder) {
                let context = self.get_context();
                <$isema>::builder(&context)
                    $(.$to(self.$from().into()))*;
            }
        }
    };
}

pub use def;
