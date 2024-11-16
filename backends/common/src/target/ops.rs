use lpl::{
    combinators::{lang::ident, literal},
    ParseResult, ParseStream, Parser,
};
use tir_core::{parser::region_with_blocks, *};
use tir_macros::{op_implements, Op, OpAssembly, OpValidator};

use crate::target::DIALECT_NAME;

#[derive(Op, Debug, Clone, OpValidator)]
#[operation(name = "section", dialect = target, known_attrs(name: String))]
pub struct SectionOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}

impl OpAssembly for SectionOp {
    fn parse_assembly(input: IRStrStream) -> ParseResult<IRStrStream, OpRef>
    where
        Self: Sized,
    {
        let parser = literal("\"")
            .and_then(ident(|c| c == '.'))
            .and_then(literal("\""))
            .flat()
            .map(|(_, name, _)| name)
            .and_then(region_with_blocks())
            .map_with(|(name, body), extra| {
                let state = extra.unwrap();
                let context = state.context();

                let section = SectionOp::builder(&context)
                    .name(name.to_string().into())
                    .body(body)
                    .build();
                let section: OpRef = section;
                section
            })
            .label("section");

        parser.parse(input)
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

#[derive(Op, Debug, OpValidator, OpAssembly)]
#[operation(name = "section_end", dialect = target)]
pub struct SectionEndOp {
    r#impl: OpImpl,
}

#[op_implements(dialect = target)]
impl Terminator for SectionEndOp {}
