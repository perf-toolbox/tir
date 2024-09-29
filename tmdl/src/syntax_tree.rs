use crate::Token;
use lpl::Spanned;

#[derive(Debug, Clone)]
pub enum CSTNode<'a> {
    InstrTemplate(Box<InstrTemplateNode<'a>>),
    Properties(Box<PropertiesNode<'a>>),
    Instr(Box<InstrNode<'a>>),
    Identifier(Spanned<&'a str>),
    // Add other node types as needed
}

#[derive(Debug, Clone)]
pub struct InstrTemplateNode<'a> {
    pub name: Spanned<&'a str>,
    pub parameters: Vec<ParameterNode<'a>>,
    pub body: Vec<CSTNode<'a>>,
}

#[derive(Debug, Clone)]
pub struct ParameterNode<'a> {
    pub name: Spanned<&'a str>,
    pub type_: TypeNode<'a>,
}

#[derive(Debug, Clone)]
pub struct TypeNode<'a> {
    pub name: Spanned<&'a str>,
    pub bit_width: Option<Spanned<i64>>,
}

#[derive(Debug, Clone)]
pub struct PropertiesNode<'a> {
    pub target: Spanned<&'a str>,
    pub properties: Vec<PropertyNode<'a>>,
}

#[derive(Debug, Clone)]
pub struct PropertyNode<'a> {
    pub name: Spanned<&'a str>,
    pub value: ExpressionNode<'a>,
}

#[derive(Debug, Clone)]
pub struct InstrNode<'a> {
    pub name: Spanned<&'a str>,
    pub template: Spanned<&'a str>,
    pub arguments: Vec<ExpressionNode<'a>>,
}

#[derive(Debug, Clone)]
pub enum ExpressionNode<'a> {
    Identifier(Spanned<&'a str>),
    StringLiteral(Spanned<&'a str>),
    IntegerLiteral(Spanned<i64>),
    BinaryOperation(Box<BinaryOperationNode<'a>>),
    // Add other expression types as needed
}

#[derive(Debug, Clone)]
pub struct BinaryOperationNode<'a> {
    pub left: Box<ExpressionNode<'a>>,
    pub operator: Spanned<Token<'a>>,
    pub right: Box<ExpressionNode<'a>>,
}
