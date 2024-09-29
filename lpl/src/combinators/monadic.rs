use std::{any::Any, rc::Rc};

use crate::{ParseStream, Parser, ParserError, Span, Spanned};

pub fn map<'a, P, F, Input, Output1, Output2>(
    parser: P,
    map_fn: F,
) -> impl Parser<'a, Input, Output2>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output1>,
    F: Fn(Output1) -> Output2,
{
    move |input: Input| {
        parser
            .parse(input)
            .map(|(result, next_input)| (map_fn(result), next_input))
    }
}

pub fn map_with<'a, P, F, Input, Output1, Output2, Extra>(
    parser: P,
    map_fn: F,
) -> impl Parser<'a, Input, Output2>
where
    Input: ParseStream<'a, Extra = Extra> + 'a,
    P: Parser<'a, Input, Output1>,
    F: Fn(Output1, Option<&Extra>) -> Output2,
{
    move |input: Input| {
        parser
            .parse(input.clone())
            .map(|(result, next_input)| (map_fn(result, input.get_extra()), next_input))
    }
}

pub fn or_else<'a, P1, P2, Input, Output>(
    parser1: P1,
    parser2: P2,
) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    P1: Parser<'a, Input, Output>,
    P2: Parser<'a, Input, Output>,
{
    move |input: Input| {
        parser1
            .parse(input.clone())
            .or_else(|_| parser2.parse(input))
    }
}

pub fn and_then<'a, P1, P2, Input, Output1, Output2>(
    parser1: P1,
    parser2: P2,
) -> impl Parser<'a, Input, (Output1, Output2)>
where
    Input: ParseStream<'a> + 'a,
    P1: Parser<'a, Input, Output1>,
    P2: Parser<'a, Input, Output2>,
{
    move |input: Input| {
        parser1
            .parse(input.clone())
            .and_then(|(out, next_input)| match next_input {
                Some(next_input) => parser2
                    .parse(next_input)
                    .map(|(out2, next_input)| ((out, out2), next_input)),
                None => Err(ParserError::new(
                    "no more input to parse".to_string(),
                    input.span(),
                )),
            })
    }
}

pub fn spanned<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Spanned<Output>>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    move |input: Input| {
        let span = input.span();
        parser.parse(input.clone()).map(|(output, next_input)| {
            let new_span = next_input
                .clone()
                .map_or(None, |input| Some(input.span().get_offset_start()));
            let final_span = Span::new(span.clone_filename(), span.get_offset_start(), new_span);
            ((output, final_span), next_input)
        })
    }
}

pub fn try_map<'a, P, F, Input, Output1, Output2>(
    parser: P,
    map_fn: F,
) -> impl Parser<'a, Input, Output2>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output1>,
    F: Fn(Output1, Span) -> Result<Output2, ParserError>,
{
    move |input: Input| {
        let span = input.span();
        let result = parser.parse(input);
        match result {
            Ok((res, next_input)) => map_fn(res, span).map(|res| (res, next_input)),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::combinators::{any_whitespace1, literal};
    use crate::parse_stream::StrStream;
    use crate::Parser;

    #[test]
    fn test_spanned() {
        let input = "Hello";
        let stream: StrStream = input.into();

        let matcher = literal("Hello").spanned();

        assert!(matcher.parse(stream).is_ok());
    }

    #[test]
    fn test_then() {
        let input = "Hello    World";
        let stream: StrStream = input.into();

        let matcher = literal("Hello")
            .and_then(any_whitespace1())
            .and_then(literal("World"));

        assert!(matcher.parse(stream.clone()).is_ok());
    }

    #[test]
    fn test_or() {
        let input = "Hello";
        let stream: StrStream = input.into();

        let matcher = literal("World").or_else(literal("Hello"));

        assert!(matcher.parse(stream.clone()).is_ok());
    }
}
