use crate::{parse_stream::ParseStream, ParseResult, Parser, ParserError};

pub fn any_char<'a, Input>(input: Input) -> ParseResult<Input, char>
where
    Input: ParseStream<'a>,
{
    if !input.is_string_like() {
        return Err(ParserError::new("Expected string-like input", input.span()));
    }

    match input.chars().next() {
        Some(next) => {
            let next_input: Option<Input> = input.slice(next.len_utf8()..input.len());
            Ok((next, next_input))
        }
        _ => Err(ParserError::new(
            "Expected a char, got end of string",
            input.span(),
        )),
    }
}

pub fn take_while<'a, Input, F>(predicate: F) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
    F: Fn(&char) -> bool,
{
    move |input: Input| {
        if !input.is_string_like() {
            return Err(ParserError::new("Expected string-like input", input.span()));
        }

        let mut last = 0;

        let chars = input.chars();

        for c in chars {
            if !predicate(&c) {
                break;
            }
            last += c.len_utf8();
        }

        if last == 0 {
            return Err(ParserError::new("", input.span()));
        }

        let next_input: Option<Input> = input.slice(last..input.len());

        let substr = input.substr(0..last).unwrap();

        Ok((substr, next_input))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StringConfig {
    pub string_separator: &'static str,
    pub allow_multiline: bool,
}

impl Default for StringConfig {
    fn default() -> Self {
        StringConfig {
            string_separator: "\"",
            allow_multiline: false,
        }
    }
}

pub fn string_literal<'a, Input>(config: StringConfig) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
{
    move |input: Input| {
        if !input.is_string_like() {
            return Err(ParserError::new("Expected string-like input", input.span()));
        }

        if !input.starts_with(config.string_separator) {
            return Err(ParserError::new(
                format!(
                    "Expected string literal to start with `{}`",
                    config.string_separator
                ),
                input.span(),
            ));
        }

        let last = || {
            let chars = input.chars().skip(config.string_separator.len());

            for (id, c) in chars.enumerate() {
                let lb = config.string_separator.len() + id;
                let ub = config.string_separator.len() * 2 + id;
                if input.substr(lb..ub).unwrap_or_default() == config.string_separator {
                    return Ok(ub);
                }

                if c == '\n' && !config.allow_multiline {
                    return Err(id);
                }
            }

            Err(input.len())
        };

        match last() {
            Ok(last) => Ok((input.substr(0..last).unwrap(), input.slice(last..))),
            Err(pos) => {
                let span = crate::Span::new(input.span().filename.clone(), pos, pos);
                Err(ParserError::new(
                    format!("Expected '{}'", config.string_separator),
                    span,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_stream::StrStream;
    use crate::Parser;

    use super::*;

    #[test]
    fn test_take_while() {
        let input = "  \tSpace";
        let with_space: StrStream = input.into();
        let input2 = "NoSpaceString";
        let no_space: StrStream = input2.into();

        let matcher = take_while(|c| c.is_whitespace());

        assert!(matcher.parse(with_space.clone()).is_ok());
        assert!(matcher.parse(no_space.clone()).is_err());
    }
}
