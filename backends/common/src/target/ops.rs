use tir_core::parser::{AsmPResult, ParseStream};
use tir_core::{parser::region_with_blocks, *};
use tir_macros::{Op, OpValidator};
use winnow::{
    ascii::{alphanumeric1, multispace0},
    combinator::delimited,
    Parser,
};

use crate::target::DIALECT_NAME;

#[derive(Op, Debug, Clone, OpValidator)]
#[operation(name = "section", known_attrs(name: String))]
pub struct SectionOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}

impl OpAssembly for SectionOp {
    fn parse_assembly(input: &mut ParseStream) -> AsmPResult<OpRef>
    where
        Self: Sized,
    {
        let (_, name, _) = delimited(
            multispace0,
            ("\"".void(), alphanumeric1, "\"".void()),
            multispace0,
        )
        .parse_next(input)?;
        let body = region_with_blocks.parse_next(input)?;
        let context = input.state.get_context();
        let section = SectionOp::builder(&context)
            .name(name.to_string().into())
            .body(body)
            .build();
        Ok(section)
    }

    fn print_assembly(&self, fmt: &mut dyn IRFormatter) {
        let name: String = self
            .get_name_attr()
            .clone()
            .try_into()
            .expect("'name' must be a string attr");
        fmt.write_direct(&format!("\"{}\" ", name));
        let body = self.get_body_region();
        print_region(fmt, &body);
    }
}
