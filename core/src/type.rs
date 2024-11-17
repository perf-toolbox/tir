use std::collections::HashMap;
use std::sync::Arc;

use lpl::combinators::lang::ident;
use lpl::combinators::literal;
use lpl::Diagnostic;
use lpl::ParseResult;
use lpl::ParseStream;
use lpl::Parser;

use crate::parser::Parsable;
use crate::Attr;
use crate::ContextRef;
use crate::ContextWRef;
use crate::DiagKind;
use crate::IRStrStream;
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
        if let Some(type_id) = dialect.get_type_id(T::get_type_name()) {
            return type_id == self.type_id;
        };

        false
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
    fn parse(input: IRStrStream) -> ParseResult<IRStrStream, Type> {
        let common_name = ident(|c| c == '_')
            .and_then(literal("."))
            .and_then(ident(|c| c == '_'))
            .map(|((dialect_name, _), type_name)| (dialect_name, type_name))
            .label("generic type name");
        let type_name = common_name.or_else(
            ident(|c| c == '_')
                .map(|type_name| ("builtin", type_name))
                .label("builtin type name"),
        );

        let parser = literal("!")
            .and_then(type_name)
            .map(|(_, (dialect, ty_name))| (dialect, ty_name))
            .label("type_name");

        let span = input.span();
        let state = input.get_extra().unwrap().clone();
        let context = state.context();
        let ((dialect_name, ty_name), next_input) = parser.parse(input)?;

        let dialect = context
            .get_dialect_by_name(dialect_name)
            .ok_or(Into::<Diagnostic>::into(DiagKind::UnknownDialect(
                dialect_name.to_string(),
                span.clone(),
            )))?;
        let id = dialect
            .get_type_id(ty_name)
            .ok_or(Into::<Diagnostic>::into(DiagKind::UnknownType(
                ty_name.to_string(),
                dialect_name.to_string(),
                span,
            )))?;

        let attr_parser = dialect.get_type_parser(id).unwrap();

        let (attrs, next_input) = attr_parser.parse(next_input.unwrap())?;

        let ty = Type::new(context, dialect.get_id(), id, attrs);

        Ok((ty, next_input))
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
