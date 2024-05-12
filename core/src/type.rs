use std::collections::HashMap;
use std::sync::Arc;

use winnow::combinator::preceded;
use winnow::Parser;

use crate::parser::{op_tuple, AsmPResult, Parsable, ParseStream};
use crate::Attr;
use crate::ContextRef;
use crate::ContextWRef;
use crate::Printable;
use crate::TyAssembly;

#[derive(Debug, Clone)]
pub struct Type {
    context: ContextWRef,
    dialect_id: u32,
    type_id: u32,
    attrs: HashMap<String, Attr>,
}

impl Type {
    pub fn new(
        context: ContextRef,
        dialect_id: u32,
        type_id: u32,
        attrs: HashMap<String, Attr>,
    ) -> Self {
        Type {
            context: Arc::downgrade(&context),
            dialect_id,
            type_id,
            attrs,
        }
    }

    pub fn get_context(&self) -> Option<ContextRef> {
        self.context.upgrade()
    }
    pub fn get_dialect_id(&self) -> u32 {
        self.dialect_id
    }

    pub fn get_type_id(&self) -> u32 {
        self.type_id
    }

    pub fn get_attrs(&self) -> &HashMap<String, Attr> {
        &self.attrs
    }

    pub fn isa<T: Ty>(&self) -> bool {
        let context = self.context.upgrade().unwrap();
        let dialect = context.get_dialect_by_name(T::get_dialect_name()).unwrap();
        if dialect.get_id() != self.dialect_id {
            return false;
        }
        let type_id = dialect.get_type_id(T::get_type_name());
        type_id == self.type_id
    }
}

impl Printable for Type {
    fn print(&self, fmt: &mut dyn crate::IRFormatter) {
        let context = self.get_context().unwrap();
        let dialect = context.get_dialect(self.dialect_id).unwrap();

        fmt.write_direct("!");
        if dialect.get_name() != crate::builtin::DIALECT_NAME {
            fmt.write_direct(&format!("{}.", dialect.get_name()));
        }

        let printer = dialect.get_type_printer(self.type_id).unwrap();
        printer(&self.attrs, fmt);
    }
}

impl Parsable<Type> for Type {
    fn parse(input: &mut ParseStream<'_>) -> AsmPResult<Type> {
        let (dialect, ty) = preceded("!", op_tuple).parse_next(input)?;

        let context = input.state.get_context();
        let dialect = context.get_dialect_by_name(dialect).unwrap();
        let id = dialect.get_type_id(ty);

        let mut parser = dialect.get_type_parser(id).unwrap();

        let attrs = parser.parse_next(input)?;

        Ok(Type::new(context, dialect.get_id(), id, attrs))
    }
}

pub trait Ty: TyAssembly {
    fn get_type_name() -> &'static str;
    fn get_dialect_name() -> &'static str;
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.context.as_ptr() == other.context.as_ptr()
            && self.type_id == other.type_id
            && self.dialect_id == other.dialect_id
            && self.attrs == other.attrs
    }
}
