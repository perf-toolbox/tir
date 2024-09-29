use core::num;

use crate::{
    combinators::{literal, text::take_while},
    parse_stream::ParseStream,
    ParseResult, Parser, ParserError,
};

/// Parses an identifier based on a custom predicate.
///
/// This function creates a parser that recognizes identifiers. An identifier
/// must start with an alphabetic character and can be followed by alphanumeric
/// characters or characters that satisfy the given predicate.
///
/// # Arguments
///
/// * `predicate` - A function that takes a `char` and returns a `bool`. This is used
///   to determine which non-alphanumeric characters are allowed in the identifier.
///
/// # Returns
///
/// A parser that, when successful, returns the parsed identifier as a string slice.
///
/// # Type Parameters
///
/// * `'a` - The lifetime of the input.
/// * `Input` - The type of the input stream, which must implement `ParseStream`.
///
/// # Errors
///
/// This parser will return an error if:
/// - The input is empty.
/// - The first character is not alphabetic.
/// - No valid identifier characters are found.
pub fn line_comment<'a, Input>(comment_start: &'static str) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
    Input::Slice: PartialEq<&'a str>,
{
    literal(comment_start)
        .and_then(take_while(|c| *c != '\n'))
        .map(|(_, c)| c)
}

pub fn ident<'a, Input>(predicate: impl Fn(char) -> bool) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
{
    move |input: Input| {
        if !input.is_string_like() {
            return Err(ParserError::new(
                "Expected string-like input".to_string(),
                input.span(),
            ));
        }

        let mut last = 0;

        let mut chars = input.chars().peekable();

        if input.len() == 0 {
            return Err(ParserError::new(
                "Expected at least one character".to_string(),
                input.span(),
            ));
        }

        if !chars.peek().unwrap().is_alphabetic() && !predicate(*chars.peek().unwrap()) {
            return Err(ParserError::new(
                "Identifier must start with an alphabetic character or satisfy a predicate"
                    .to_string(),
                input.span(),
            ));
        }

        for c in chars {
            if !c.is_alphanumeric() && !predicate(c) {
                break;
            }
            last += c.len_utf8();
        }

        if last == 0 {
            return Err(ParserError::new(
                "Expected identifier".to_string(),
                input.span(),
            ));
        }

        let next_input: Option<Input> = input.slice(last..input.len());

        let substr = input.substr(0..last).unwrap();

        Ok((substr, next_input))
    }
}

pub trait Integer {
    fn parse_int(input: &str, radix: u32) -> Result<Self, std::num::ParseIntError>
    where
        Self: Sized;
}

impl Integer for i64 {
    fn parse_int(input: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(input, radix)
    }
}

pub fn integer_literal<'a, Input, Output>(radix: u32) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    Output: Integer,
{
    move |input: Input| {
        let chars = input.chars();
        let mut last = 0;

        for c in chars {
            if !c.is_digit(radix) {
                break;
            }
            last += c.len_utf8();
        }

        let next_input: Option<Input> = input.slice(last..input.len());

        let substr = input.substr(0..last).unwrap();

        let parsed_int = Output::parse_int(substr, radix);

        if let Ok(parsed_int) = parsed_int {
            Ok((parsed_int, next_input))
        } else {
            Err(ParserError::new(
                "Expected integer literal".to_string(),
                input.span(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser, StrStream};
    #[test]
    fn test_line_comment() {
        let input = "// This is a comment\n";
        let stream: StrStream = input.into();
        let parser = line_comment("//");
        let result = parser.parse(stream);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " This is a comment");
    }
}
