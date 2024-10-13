use crate::{InternalError, ParseResult, ParseStream, Parser};

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
            Err(InternalError::ExpectedEof(input.span()).into())
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

pub fn isolate_block<'a, P, SP, EP, Input, Output>(
    start: SP,
    end: EP,
    parser: P,
) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    SP: Parser<'a, Input, ()>,
    EP: Parser<'a, Input, ()>,
    P: Parser<'a, Input, Output>,
{
    move |input: Input| {
        let mut depth = 1;

        let mut next_input: Option<Input>;

        match start.parse(input.clone()) {
            Ok((_, ni)) => {
                next_input = ni;
            }
            Err(err) => return Err(err),
        }

        while depth > 0 && next_input.is_some() {
            if let Ok((_, ni)) = start.parse(next_input.as_ref().cloned().unwrap()) {
                depth += 1;
                next_input = ni;
            } else if let Ok((_, ni)) = end.parse(next_input.as_ref().cloned().unwrap()) {
                depth -= 1;
                next_input = ni;
            } else {
                next_input = next_input.unwrap().slice(1..);
            }
        }

        if depth != 0 {
            return Err(InternalError::UnmatchedPair(input.span()).into());
        }

        let span = input.span();

        let isolated_input = if let Some(ref ni) = next_input {
            let offset = input.len() - ni.len();
            input.slice(..offset).unwrap()
        } else {
            input
        };

        let (result, remainder) = parser.parse(isolated_input)?;

        if remainder.map(|r| r.len()).unwrap_or_default() > 0 {
            return Err(InternalError::ExpectedEof(span).into());
        }

        Ok((result, next_input))
    }
}

pub fn isolate_until<'a, P1, P2, Input, Output>(
    pred: P1,
    parser: P2,
) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    P1: Parser<'a, Input, ()>,
    P2: Parser<'a, Input, Output>,
{
    move |input: Input| {
        let mut next_input: Option<Input> = Some(input.clone());

        let mut found = false;
        while next_input.is_some() {
            if pred.parse(next_input.as_ref().cloned().unwrap()).is_ok() {
                found = true;
                break;
            }

            next_input = next_input.unwrap().slice(1..);
        }

        if next_input.is_none() && !found {
            return Err(InternalError::PredNotSatisfied(input.span()).into());
        }

        let span = input.span();
        let isolated = if let Some(ref ni) = next_input {
            let offset = input.len() - ni.len();
            input.slice(0..offset).unwrap()
        } else {
            input
        };

        let (result, remainder) = parser.parse(isolated)?;

        if remainder.map(|r| r.len()).unwrap_or_default() > 0 {
            return Err(InternalError::ExpectedEof(span).into());
        }

        Ok((result, next_input))
    }
}
