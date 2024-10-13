use std::ops::RangeBounds;

use lpl::{
    syntax::{
        GreenElement, GreenNode, GreenNodeData, GreenToken, GreenTokenData, RedElement, RedNode,
        RedNodeData,
    },
    ParseStream, Span,
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
pub enum SyntaxKind {
    Eof,

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

    // Operators
    /// `=`
    Equals,

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

    StructFieldAccess,

    EncodingDecl,
    AsmDecl,

    BlockExpr,
    LiteralExpr,
    BinOpExpr,

    EnumDecl,
    EnumBody,
    EnumVariantDecl,
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    tokens: &'a [GreenElement<SyntaxKind>],
    offset: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [GreenElement<SyntaxKind>]) -> Self {
        Self { tokens, offset: 0 }
    }
}

impl<'a> ParseStream<'a> for TokenStream<'a> {
    type Slice = &'a [GreenElement<SyntaxKind>];

    type Extra = ();

    type Item = GreenElement<SyntaxKind>;

    fn get<R>(&self, range: R) -> Option<Self::Slice>
    where
        R: RangeBounds<usize>,
    {
        self.tokens
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
    }

    fn slice<R>(&self, range: R) -> Option<Self>
    where
        Self: Sized,
        R: RangeBounds<usize>,
    {
        let offset = match range.start_bound() {
            std::ops::Bound::Included(bound) => self.offset + bound,
            std::ops::Bound::Excluded(bound) => self.offset + bound + 1,
            std::ops::Bound::Unbounded => self.offset,
        };
        self.tokens
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
            .map(|tokens| Self { tokens, offset })
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn span(&self) -> lpl::Span {
        self.tokens
            .first()
            .map(|t| t.as_token().span())
            .unwrap_or(Span::empty())
    }

    fn set_extra(&mut self, _extra: Self::Extra) {
        todo!()
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        todo!()
    }

    fn peek(&self) -> Option<Self::Item> {
        todo!()
    }

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.tokens.get(n).cloned()
    }
}
