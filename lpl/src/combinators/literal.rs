use crate::{InternalError, ParseStream, Parser};

pub fn literal<'a, Input: ParseStream<'a> + 'a>(
    expected: &'static str,
) -> impl Parser<'a, Input, &'a str>
where
    Input::Slice: PartialEq<&'a str>,
{
    let parser = move |input: Input| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((expected, input.slice(expected.len()..input.len()))),
        _ => Err(InternalError::ExpectedNotFound(expected, input.span()).into()),
    };

    parser.label(expected)
}

#[cfg(test)]
mod tests {
    use crate::parse_stream::StrStream;
    use crate::Parser;

    use super::literal;

    #[test]
    fn match_literal() {
        let input = "Hello World";
        let stream: StrStream = input.into();

        let hello_matcher = literal("Hello");
        let random_matcher = literal("none");

        assert!(hello_matcher.parse(stream.clone()).is_ok());
        assert!(random_matcher.parse(stream).is_err());
    }
}
