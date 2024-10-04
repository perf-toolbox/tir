use crate::{ParseResult, ParseStream, Parser, ParserError};

pub fn todo<'a, Input, Output>() -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    move |_| -> ParseResult<Input, Output> { todo!() }
}

pub fn eof<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: ParseStream<'a> + 'a,
{
    move |input: Input| -> ParseResult<Input, ()> {
        if input.len() == 0 {
            Ok(((), Some(input)))
        } else {
            Err(ParserError::new("Expected end of input", input.span()))
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
