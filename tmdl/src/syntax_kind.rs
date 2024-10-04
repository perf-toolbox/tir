use lpl::syntax::{GreenToken, GreenTokenData};

pub type Token = GreenToken<SyntaxKind>;
pub type TokenData = GreenTokenData<SyntaxKind>;

/// A piece of syntax in the TMDL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    // Keywords
    /// `instr_template`
    InstrTemplateKw,
    /// `properties`
    PropertiesKw,
    /// `for`
    ForKw,
    /// `let`
    LetKw,
    /// `instr`
    InstrKw,

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

    InstrTemplate,
}
