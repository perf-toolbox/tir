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

pub fn optional<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Option<Output>>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    move |input: Input| {
        if let Ok((output, next_input)) = parser.parse(input.clone()) {
            Ok((Some(output), next_input))
        } else {
            Ok((None, Some(input)))
        }
    }
}

pub fn fold_left<'a, A, O, Input, Output1, Output2, Acc>(
    atom: A,
    operator: O,
    acc: Acc,
) -> impl Parser<'a, Input, Output1>
where
    Input: ParseStream<'a> + 'a,
    A: Parser<'a, Input, Output1>,
    O: Parser<'a, Input, Output2>,
    Acc: Fn(Output1, Output2, Output1) -> Output1,
{
    move |input: Input| {
        let (mut result, mut next_input) = atom.parse(input.clone())?;

        loop {
            if let Some(ref next_input_unwrapped) = next_input {
                match operator.parse(next_input_unwrapped.clone()) {
                    Ok((op, op_next_input)) => {
                        match atom.parse(op_next_input.unwrap_or(input.clone())) {
                            Ok((right, right_next_input)) => {
                                result = acc(result, op, right);
                                next_input = right_next_input;
                            }
                            Err(_) => break,
                        }
                    }
                    Err(_) => break,
                }
            } else {
                break;
            }
        }

        Ok((result, next_input))
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
