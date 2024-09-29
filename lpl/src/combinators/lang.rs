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

        if !chars.peek().unwrap().is_alphabetic() {
            return Err(ParserError::new(
                "Identifier must start with an alphabetic character".to_string(),
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
            return Err(ParserError::new("".to_string(), input.span()));
        }

        let next_input: Option<Input> = input.slice(last..input.len());

        let substr = input.substr(0..last).unwrap();

        Ok((substr, next_input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParseResult, Parser, StrStream};
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
