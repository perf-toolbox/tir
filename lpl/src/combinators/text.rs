use crate::{parse_stream::ParseStream, InternalError, ParseResult, Parser};

use super::{literal, optional};

pub fn any_char<'a, Input>(input: Input) -> ParseResult<Input, char>
where
    Input: ParseStream<'a>,
{
    if !input.is_string_like() {
        return Err(InternalError::NotStringLike(input.span()).into());
    }

    match input.chars().next() {
        Some(next) => {
            let next_input: Option<Input> = input.slice(next.len_utf8()..input.len());
            Ok((next, next_input))
        }
        _ => Err(InternalError::UnexpectedEof(input.span()).into()),
    }
}

pub fn take_while<'a, Input, F>(predicate: F) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
    F: Fn(&char) -> bool,
{
    move |input: Input| {
        if !input.is_string_like() {
            return Err(InternalError::NotStringLike(input.span()).into());
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
            return Err(InternalError::PredNotSatisfied(input.span()).into());
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
            return Err(InternalError::NotStringLike(input.span()).into());
        }

        if !input.starts_with(config.string_separator) {
            return Err(
                InternalError::UnexpectedPrefix(config.string_separator, input.span()).into(),
            );
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
                Err(InternalError::UnexpectedEof(span).into())
            }
        }
    }
}

pub fn ident<'a, Input>(predicate: impl Fn(char) -> bool) -> impl Parser<'a, Input, &'a str>
where
    Input: ParseStream<'a> + 'a,
{
    move |input: Input| {
        if !input.is_string_like() {
            return Err(InternalError::NotStringLike(input.span()).into());
        }

        let mut last = 0;

        let mut chars = input.chars().peekable();

        if input.len() == 0 {
            return Err(InternalError::UnexpectedEof(input.span()).into());
        }

        if !chars.peek().unwrap().is_alphabetic() {
            return Err(InternalError::NotAlpha(input.span()).into());
        }

        for c in chars {
            if !c.is_alphanumeric() && !predicate(c) {
                break;
            }
            last += c.len_utf8();
        }

        if last == 0 {
            return Err(InternalError::PredNotSatisfied(input.span()).into());
        }

        let next_input: Option<Input> = input.slice(last..input.len());

        let substr = input.substr(0..last).unwrap();

        Ok((substr, next_input))
    }
}

pub fn dec_number<'a, Input, Int>() -> impl Parser<'a, Input, Int>
where
    Input: ParseStream<'a> + 'a,
    Input::Slice: PartialEq<&'a str>,
    Int: std::str::FromStr + 'a,
{
    optional(literal("-"))
        .and_then(take_while(|c| c.is_ascii_digit()))
        .map(|(sign, num)| {
            let num = if sign.is_some() {
                &format!("-{}", num)
            } else {
                num
            };
            if let Ok(num) = Int::from_str(num) {
                num
            } else {
                unreachable!()
            }
        })
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
