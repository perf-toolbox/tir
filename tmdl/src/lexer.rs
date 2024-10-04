use lpl::combinators::text::take_while;
use lpl::combinators::{
    eof,
    lang::{ident, integer_literal, line_comment},
    literal, zero_or_more,
};
use lpl::syntax::GreenElement;
use lpl::Parser;
use lpl::ParserError;
use lpl::StrStream;

use crate::{SyntaxKind, Token, TokenData};

/// Lexes the input string into a stream of tokens.
///
/// This function takes a string input and attempts to tokenize it using various lexing functions.
/// It processes the input in the following order:
/// 1. Keywords
/// 2. Punctuation
/// 3. Built-in types
/// 4. Operators
/// 5. Identifiers
/// 6. Comments
///
/// # Arguments
///
/// * `input` - A string slice that holds the input to be lexed.
///
/// # Returns
///
/// * `Result<TokenStream<'a>, ParserError>` - A Result containing either a TokenStream on success,
///   or a ParserError if lexing fails.
pub fn lex<'src>(input: &'src str) -> Result<Vec<GreenElement<SyntaxKind>>, ParserError> {
    let stream: StrStream = input.into();

    let token = lex_keyword()
        .or_else(lex_bit_literal())
        .or_else(lex_identifier())
        .or_else(lex_punctuation())
        .or_else(lex_integer_literal())
        .or_else(lex_comment())
        .or_else(lex_whitespace());

    let parser = zero_or_more(
        token
            .spanned()
            .map(|(t, s)| GreenElement::Token(t.spanned(s))),
    )
    .and_then(eof());

    let ((tokens, _), _) = parser.parse(stream)?;

    Ok(tokens)
}

fn lex_integer_literal<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    integer_literal(10).map(|n| TokenData::new(SyntaxKind::IntegerLiteral, n.to_owned()))
}

fn lex_bit_literal<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    literal("0b")
        .and_then(take_while(|c| *c == '1' || *c == '0'))
        .map(|(prefix, bits)| TokenData::new(SyntaxKind::BitLiteral, format!("{}{}", prefix, bits)))
}

fn lex_keyword<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    literal("instr_template")
        .map(|kw| TokenData::new(SyntaxKind::InstrTemplateKw, kw.to_owned()))
        .or_else(
            literal("properties").map(|kw| TokenData::new(SyntaxKind::PropertiesKw, kw.to_owned())),
        )
        .or_else(literal("instr").map(|kw| TokenData::new(SyntaxKind::InstrKw, kw.to_owned())))
        .or_else(literal("for").map(|kw| TokenData::new(SyntaxKind::ForKw, kw.to_owned())))
        .or_else(literal("let").map(|kw| TokenData::new(SyntaxKind::LetKw, kw.to_owned())))
}

fn lex_punctuation<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    literal("{")
        .map(|p| TokenData::new(SyntaxKind::LeftBrace, p.to_owned()))
        .or_else(literal("}").map(|p| TokenData::new(SyntaxKind::RightBrace, p.to_owned())))
        .or_else(literal("<").map(|p| TokenData::new(SyntaxKind::LeftAngle, p.to_owned())))
        .or_else(literal(">").map(|p| TokenData::new(SyntaxKind::RightAngle, p.to_owned())))
        .or_else(literal(":").map(|p| TokenData::new(SyntaxKind::Colon, p.to_owned())))
        .or_else(literal(";").map(|p| TokenData::new(SyntaxKind::Semicolon, p.to_owned())))
        .or_else(literal(",").map(|p| TokenData::new(SyntaxKind::Comma, p.to_owned())))
        .or_else(literal(".").map(|p| TokenData::new(SyntaxKind::Dot, p.to_owned())))
        .or_else(literal("$").map(|p| TokenData::new(SyntaxKind::DollarSign, p.to_owned())))
        .or_else(literal("@").map(|p| TokenData::new(SyntaxKind::At, p.to_owned())))
        .or_else(literal("\"").map(|p| TokenData::new(SyntaxKind::DoubleQuote, p.to_owned())))
        .or_else(literal("=").map(|p| TokenData::new(SyntaxKind::Equals, p.to_owned())))
}

fn lex_identifier<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    ident(|c| c == '_' || c == '$').map(|i| TokenData::new(SyntaxKind::Identifier, i.to_string()))
}

fn lex_comment<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    line_comment("//").map(|c| TokenData::new(SyntaxKind::Comment, c.to_string()))
}

fn lex_whitespace<'a>() -> impl Parser<'a, StrStream<'a>, Token> {
    take_while(|c| c.is_whitespace()).map(|p| TokenData::new(SyntaxKind::Whitespace, p.to_string()))
}
