use lpl::combinators::{
    lang::{ident, line_comment},
    literal, zero_or_more,
};
use lpl::Parser;
use lpl::ParserError;
use lpl::StrStream;
use lpl::{Span, Spanned};

use crate::Token;

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
pub fn lex<'a>(input: &'a str) -> Result<Vec<Spanned<Token<'a>>>, ParserError> {
    let stream: StrStream = input.into();

    let token = lex_keyword()
        .or_else(lex_punctuation())
        .or_else(lex_builtin_type())
        .or_else(lex_operator())
        .or_else(lex_identifier())
        .or_else(lex_comment());

    let parser = zero_or_more(token.spanned());

    let (tokens, _) = parser.parse(stream)?;

    Ok(tokens)
}

fn lex_keyword<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    literal("instr_template")
        .map(|_| Token::InstrTemplate)
        .or_else(literal("properties").map(|_| Token::Properties))
        .or_else(literal("instr").map(|_| Token::Instr))
        .or_else(literal("for").map(|_| Token::For))
        .or_else(literal("let").map(|_| Token::Let))
}

fn lex_punctuation<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    literal("{")
        .map(|_| Token::LeftBrace)
        .or_else(literal("}").map(|_| Token::RightBrace))
        .or_else(literal("<").map(|_| Token::LeftAngle))
        .or_else(literal(">").map(|_| Token::RightAngle))
        .or_else(literal(":").map(|_| Token::Colon))
        .or_else(literal(";").map(|_| Token::Semicolon))
        .or_else(literal(",").map(|_| Token::Comma))
        .or_else(literal("$").map(|_| Token::Dollar))
        .or_else(literal("@").map(|_| Token::At))
}

fn lex_builtin_type<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    literal("Register")
        .map(|_| Token::Register)
        .or_else(literal("bits").map(|_| Token::Bits))
        .or_else(literal("str").map(|_| Token::Str))
}

fn lex_operator<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    literal("=").map(|_| Token::Equals)
}

fn lex_identifier<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    ident(|c| c == '_').map(Token::Identifier)
}

fn lex_comment<'a>() -> impl Parser<'a, StrStream<'a>, Token<'a>> {
    line_comment("//").map(Token::Comment)
}
