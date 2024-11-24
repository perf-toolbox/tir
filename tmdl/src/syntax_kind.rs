use core::fmt;

use lpl::syntax::{
    GreenElement, GreenNode, GreenNodeData, GreenToken, GreenTokenData, NodeOrToken, RedElement,
    RedNode, RedNodeData, SyntaxLike,
};

pub type Token = GreenToken<SyntaxKind>;
pub type TokenData = GreenTokenData<SyntaxKind>;
pub type ImmNode = GreenNode<SyntaxKind>;
pub type ImmNodeData = GreenNodeData<SyntaxKind>;
pub type ImmElement = GreenElement<SyntaxKind>;
pub type SyntaxElement = RedElement<SyntaxKind>;
pub type SyntaxNode = RedNode<SyntaxKind>;
pub type SyntaxNodeData = RedNodeData<SyntaxKind>;

/// A piece of syntax in the TMDL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SyntaxKind {
    Invalid = 0,

    // Keywords
    /// `instr_template`
    InstrTemplateKw,
    /// `encoding`
    EncodingKw,
    /// `asm`
    AsmKw,
    /// `for`
    ForKw,
    /// `let`
    LetKw,
    /// `instr`
    InstrKw,
    /// `enum`
    EnumKw,
    /// `impl`
    ImplKw,
    /// `self`
    SelfKw,
    /// `fn`
    FnKw,
    /// `flag`
    FlagKw,

    // Identifiers and literals
    Identifier,
    StringLiteral,
    IntegerLiteral,
    BitLiteral,

    // Symbols
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `<`
    LeftAngle,
    /// `>`
    RightAngle,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `$`
    DollarSign,
    /// `@`
    At,
    /// `"`
    DoubleQuote,
    /// `#`
    Pound,
    /// '-'
    Minus,

    // Compound symbols
    /// '->'
    Arrow,

    // Operators
    /// `=`
    Equals,

    LocalDocComment,
    Comment,

    Whitespace,

    TranslationUnit,

    InstrTemplateDecl,
    InstrTemplateName,
    InstrTemplateParams,
    InstrTemplateSingleParam,
    InstrTemplateSingleParamName,
    InstrTemplateBody,

    InstrDecl,
    InstrName,
    InstrParentTemplate,
    InstrParentTemplateName,
    InstrParentTemplateArg,

    Type,
    TypeParams,

    StructBody,
    StructField,
    StructFieldName,

    ImplDecl,
    ImplBody,
    ImplTraitName,
    ImplTargetName,

    EncodingDecl,
    AsmDecl,

    FlagDecl,

    ListExpr,
    BlockExpr,
    LiteralExpr,
    FieldExpr,
    BinOpExpr,
    BinOpExprLeft,
    BinOpExprRight,
    BinOpExprOp,

    ExprStmt,

    FnDecl,
    FnSignature,
    FnParamList,
    FnParam,
    FnRetType,

    EnumDecl,
    EnumBody,
    EnumVariantDecl,

    AttrList,
    Attr,
}

impl fmt::Display for SyntaxKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SyntaxLike for SyntaxKind {
    fn is_trivia(&self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
    }
}

impl PartialEq<SyntaxKind> for GreenElement<SyntaxKind> {
    fn eq(&self, other: &SyntaxKind) -> bool {
        match self {
            NodeOrToken::Token(t) => &t.kind() == other,
            NodeOrToken::Node(n) => &n.kind() == other,
        }
    }
}

pub type TokenStream<'a> = lpl::TokenStream<'a, SyntaxKind>;
