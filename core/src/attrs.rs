use crate::{Printable, Type};

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

#[derive(Debug, Clone)]
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
            Attr::String(value) => fmt.write_direct(&format!("<str: {}>", &value)),
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
