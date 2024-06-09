use winnow::{
    ascii::space0, combinator::{delimited, separated_pair}, token::take_till, Parser
};

use crate::{
    parser::{identifier, AsmPResult, Parsable, ParseStream},
    Printable, Type,
};

macro_rules! impl_from {
    ($case:ident, $from:ty) => {
        impl From<$from> for Attr {
            fn from(value: $from) -> Self {
                Attr::$case(value)
            }
        }

        impl TryInto<$from> for Attr {
            type Error = ();
            fn try_into(self) -> Result<$from, Self::Error> {
                if let Attr::$case(value) = self {
                    return Ok(value);
                }

                Err(())
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    String(String),
    Bool(bool),
    I8(i8),
    U8(u8),
    I8Array(Vec<i8>),
    U8Array(Vec<u8>),
    I16(i16),
    U16(u16),
    I16Array(Vec<i16>),
    U16Array(Vec<u16>),
    I32(i32),
    U32(u32),
    I32Array(Vec<i32>),
    U32Array(Vec<u32>),
    I64(i64),
    U64(u64),
    I64Array(Vec<i64>),
    U64Array(Vec<u64>),
    Type(Type),
    TypeArray(Vec<Type>),
}

impl Printable for Attr {
    fn print(&self, fmt: &mut dyn crate::IRFormatter) {
        match self {
            Attr::String(value) => fmt.write_direct(&format!("<str: \"{}\">", &value)),
            Attr::Bool(value) => fmt.write_direct(&format!("<bool: {}>", &value)),
            Attr::I8(value) => fmt.write_direct(&format!("<i8: {}>", &value)),
            Attr::U8(value) => fmt.write_direct(&format!("<u8: {}>", &value)),
            Attr::I16(value) => fmt.write_direct(&format!("<i16: {}>", &value)),
            Attr::U16(value) => fmt.write_direct(&format!("<u16: {}>", &value)),
            Attr::I32(value) => fmt.write_direct(&format!("<i32: {}>", &value)),
            Attr::U32(value) => fmt.write_direct(&format!("<u32: {}>", &value)),
            Attr::I64(value) => fmt.write_direct(&format!("<i64: {}>", &value)),
            Attr::U64(value) => fmt.write_direct(&format!("<u64: {}>", &value)),
            _ => todo!(),
        }
    }
}

impl Parsable<Attr> for Attr {
    fn parse(input: &mut ParseStream<'_>) -> AsmPResult<Attr> {
        let value_parser = take_till(1.., |c| c == '>');
        let atom = separated_pair(identifier, (space0, ":", space0), value_parser);
        let (ty, value) =
            delimited((space0, "<", space0), atom, (space0, ">", space0)).parse_next(input)?;
        let value = value.trim();

        // TODO error handling for String attrs
        match ty {
            "str" => Ok(Attr::String(value.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().to_string())),
            "i8" => Ok(Attr::I8(value.parse::<i8>().unwrap())),
            "u8" => Ok(Attr::U8(value.parse::<u8>().unwrap())),
            "i16" => Ok(Attr::I16(value.parse::<i16>().unwrap())),
            "u16" => Ok(Attr::U16(value.parse::<u16>().unwrap())),
            "i32" => Ok(Attr::I32(value.parse::<i32>().unwrap())),
            "u32" => Ok(Attr::U32(value.parse::<u32>().unwrap())),
            "i64" => Ok(Attr::I64(value.parse::<i64>().unwrap())),
            "u64" => Ok(Attr::U64(value.parse::<u64>().unwrap())),
            _ => todo!(),
        }
    }
}

impl_from!(String, String);
impl_from!(Bool, bool);
impl_from!(I8, i8);
impl_from!(U8, u8);
impl_from!(I8Array, Vec<i8>);
impl_from!(U8Array, Vec<u8>);
impl_from!(I16, i16);
impl_from!(U16, u16);
impl_from!(I16Array, Vec<i16>);
impl_from!(U16Array, Vec<u16>);
impl_from!(I32, i32);
impl_from!(U32, u32);
impl_from!(I32Array, Vec<i32>);
impl_from!(U32Array, Vec<u32>);
impl_from!(I64, i64);
impl_from!(U64, u64);
impl_from!(I64Array, Vec<i64>);
impl_from!(U64Array, Vec<u64>);
impl_from!(Type, Type);
impl_from!(TypeArray, Vec<Type>);
