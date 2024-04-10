use crate::Type;

#[derive(Debug, Clone)]
pub enum Attr {
    String(String),
    Int(i64),
    Bool(bool),
    Type(Type),
    TypeArray(Vec<Type>),
}
