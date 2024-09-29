use lpl::Spanned;

#[derive(Debug, Clone)]
pub enum CSTNode {
    InstrTemplate(Box<InstrTemplateNode>),
    Properties(Box<PropertiesNode>),
    Instr(Box<InstrNode>),
    Identifier(Spanned<String>),
    Comment(Spanned<String>),
    StructField(StructFieldNode),
    // Add other node types as needed
}

#[derive(Debug, Clone)]
pub struct InstrTemplateNode {
    pub name: Spanned<String>,
    pub parameters: Vec<TemplateParameterNode>,
    pub body: Vec<CSTNode>,
}

#[derive(Debug, Clone)]
pub struct StructFieldNode {
    pub name: Spanned<String>,
    pub type_: TypeNode,
}

#[derive(Debug, Clone)]
pub struct TemplateParameterNode {
    pub name: Spanned<String>,
    pub type_: TypeNode,
}

#[derive(Debug, Clone)]
pub enum TypeNode {
    Register,
    Bits(Spanned<i64>),
    Str,
}

#[derive(Debug, Clone)]
pub struct PropertiesNode {
    pub target: Spanned<String>,
    pub properties: Vec<PropertyNode>,
}

#[derive(Debug, Clone)]
pub struct PropertyNode {
    pub name: Spanned<String>,
    pub value: ExpressionNode,
}

#[derive(Debug, Clone)]
pub struct InstrNode {
    pub name: Spanned<String>,
    pub template: Spanned<String>,
    pub arguments: Vec<ExpressionNode>,
}

#[derive(Debug, Clone)]
pub enum ExpressionNode {
    StringLiteral(Spanned<String>),
    IntegerLiteral(Spanned<i64>),
    BinaryOperation(Box<BinaryOperationNode>),
    Path(Vec<Spanned<String>>),
    // Add other expression types as needed
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    BitConcat,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct BinaryOperationNode {
    pub left: Box<ExpressionNode>,
    pub operator: BinaryOperator,
    pub right: Box<ExpressionNode>,
}
