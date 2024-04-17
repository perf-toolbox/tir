use crate::Type;

macro_rules! impl_from {
    ($case:ident, $from:ty) => {
        impl From<$from> for Attr {
            fn from(value: $from) -> Self {
                Attr::$case(value)
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
