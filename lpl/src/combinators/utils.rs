use crate::{ParseResult, ParseStream, Parser};

pub fn todo<'a, Input, Output>() -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    move |input| -> ParseResult<Input, Output> { todo!() }
}
