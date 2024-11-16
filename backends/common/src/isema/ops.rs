use lpl::{ParseResult, ParseStream, Parser};
use tir_core::{
    parser::single_block_region, IRFormatter, IRStrStream, Op, OpAssembly, OpImpl, OpRef,
    Printable, RegionRef, Terminator,
};
use tir_macros::{op_implements, Op, OpAssembly, OpValidator};

use crate::isema::DIALECT_NAME;

/// A compound instruction
///
/// Sometimes operations can do multiple things at once. To allow one model complex instructions
/// and still preserve operation atomicity, introduce a container operation, that can represent
/// instructions as a combination of simpler operations.
#[derive(Op, Debug, Clone, OpValidator)]
#[operation(name = "comp_instr", dialect = isema, known_attrs(asm: String))]
pub struct CompInstrOp {
    #[region(single_block, no_args)]
    body: RegionRef,
    r#impl: OpImpl,
}

/// Terminator for compound instructions
#[derive(Op, Debug, Clone, OpAssembly, OpValidator)]
#[operation(name = "comp_instr_end", dialect = isema)]
pub struct CompInstrEndOp {
    r#impl: OpImpl,
}

#[op_implements(dialect = isema)]
impl Terminator for CompInstrEndOp {}

impl OpAssembly for CompInstrOp {
    fn parse_assembly(input: IRStrStream) -> ParseResult<IRStrStream, OpRef>
    where
        Self: Sized,
    {
        let parser = single_block_region().map_with(|ops, extra| {
            let state = extra.unwrap();
            let context = state.context();

            let comp = CompInstrOp::builder(&context).build();
            for op in ops {
                comp.borrow_mut().get_body().push(&op);
            }

            let comp: OpRef = comp;
            comp
        });

        parser.parse(input)
    }

    fn print_assembly(&self, fmt: &mut dyn IRFormatter) {
        fmt.start_region();
        let body = self.get_body();
        for op in body.iter() {
            op.borrow().print(fmt);
        }
        fmt.end_region();
    }
}

/// Load data from memory to register
#[derive(Op, Debug, Clone, OpAssembly, OpValidator)]
#[operation(
    name = "load",
    dialect = isema,
    known_attrs(dst: String, base_addr: String, offset: i16, width: u8, sign_extend: bool)
)]
pub struct LoadOp {
    r#impl: OpImpl,
}

/// Store data from register to memory
#[derive(Op, Debug, Clone, OpAssembly, OpValidator)]
#[operation(
    name = "store",
    dialect = isema,
    known_attrs(src: String, base_addr: String, offset: i16, width: u8)
)]
pub struct StoreOp {
    r#impl: OpImpl,
}

// Three-register operations

macro_rules! three_reg_ops {
    ($($struct_name:ident => { name = $op_name:literal, doc = $doc:literal })*) => {
        $(
            #[doc = $doc]
            #[derive(Op, Debug, Clone, OpAssembly, OpValidator)]
            #[operation(
                name = $op_name,
                dialect = isema,
                known_attrs(rs1: String, rs2: Option<String>, imm: Option<I16Attr>, rd: String, width: u8)
            )]
            pub struct $struct_name {
                r#impl: OpImpl,
            }
        )*
    };
}

three_reg_ops! {
    AddOp => {name = "add", doc = "Compute rs1 + rs2 and store result to rd"}
    SubOp => {name = "sub", doc = "Compute rs1 - rs2 and store result to rd"}
    AndOp => {name = "and", doc = "Compute bitwise rs1 `and` rs2 and store result to rd"}
    OrOp => {name = "or", doc = "Compute bitwise rs1 `or` rs2 and store result to rd"}
    XorOp => {name = "xor", doc = "Compute bitwise rs1 `xor` rs2 and store result to rd"}
    SllOp => {name = "sll", doc = "Compute shift left logical rs1 << rs2 and store result to rd"}
    SrlOp => {name = "srl", doc = "Compute shift right logical rs1 >> rs2 and store result to rd"}
    SraOp => {name = "sra", doc = "Compute shift right arithmetic rs1 >> rs2 and store result to rd"}
}
