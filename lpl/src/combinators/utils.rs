use crate::{InternalError, ParseResult, ParseStream, Parser};

pub fn todo<'a, Input, Output>() -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    move |_| -> ParseResult<Input, Output> { todo!() }
}

pub fn eof<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Output>
where
    P: Parser<'a, Input, Output> + 'a,
    Input: ParseStream<'a> + 'a,
{
    move |input: Input| -> ParseResult<Input, Output> {
        let (res, ni) = parser.parse(input)?;
        if let Some(ni) = ni {
            Err(InternalError::ExpectedEof(ni.span()).into())
        } else {
            Ok((res, None))
        }
    }
}

pub fn reset<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    move |input: Input| {
        parser
            .parse(input.clone())
            .map(|(output, _)| (output, Some(input)))
    }
}
