use crate::{InternalError, ParseStream, Parser};

pub fn one_or_more<'a, P, Input: ParseStream<'a> + 'a, Output>(
    parser: P,
) -> impl Parser<'a, Input, Vec<Output>>
where
    P: Parser<'a, Input, Output>,
{
    move |input: Input| {
        let mut result = Vec::new();

        let mut next_input: Option<Input>;

        match parser.parse(input.clone()) {
            Ok((first_item, ni)) => {
                next_input = ni;
                result.push(first_item);
            }
            Err(err) => {
                return Err(err);
            }
        }

        while let Some(ref inp) = next_input {
            if let Ok((next_item, ni)) = parser.parse(inp.clone()) {
                next_input = ni;
                result.push(next_item);
            } else {
                break;
            }
        }

        Ok((result, next_input))
    }
}

pub fn zero_or_more<'a, P, Input: ParseStream<'a> + 'a, Output>(
    parser: P,
) -> impl Parser<'a, Input, Vec<Output>>
where
    P: Parser<'a, Input, Output>,
{
    move |input| {
        let mut result = Vec::new();

        let mut next_input: Option<Input> = Some(input);

        while let Some(ref inp) = next_input {
            if let Ok((next_item, ni)) = parser.parse(inp.clone()) {
                next_input = ni;
                result.push(next_item);
            } else {
                break;
            }
        }

        Ok((result, next_input))
    }
}

pub fn interleaved<'a, P1, P2, Input: ParseStream<'a> + 'a, Output>(
    parser: P1,
    ignored: P2,
) -> impl Parser<'a, Input, Vec<Output>>
where
    P1: Parser<'a, Input, Output>,
    P2: Parser<'a, Input, ()>,
{
    move |input| {
        let mut result = Vec::new();

        let mut next_input: Option<Input> = Some(input);

        while let Some(ref inp) = next_input {
            if let Ok((_, ni)) = ignored.parse(inp.clone()) {
                next_input = ni;
                continue;
            }

            if let Ok((next_item, ni)) = parser.parse(inp.clone()) {
                next_input = ni;
                result.push(next_item);
            } else {
                break;
            }
        }

        Ok((result, next_input))
    }
}

pub fn separated_ignore<'a, P1, P2, Input, Output>(
    parser: P1,
    ignored: P2,
) -> impl Parser<'a, Input, Vec<Output>>
where
    Input: ParseStream<'a> + 'a,
    P1: Parser<'a, Input, Output>,
    P2: Parser<'a, Input, ()>,
{
    move |input: Input| {
        let mut result = Vec::new();

        let mut next_input: Option<Input> = Some(input.clone());

        while let Some(ref inp) = next_input.clone() {
            if let Ok((next_item, ni)) = parser.parse(inp.clone()) {
                next_input = ni;
                result.push(next_item);
            } else {
                break;
            }

            if let Some(ref inp) = next_input.clone() {
                if let Ok((_, ni)) = ignored.parse(inp.clone()) {
                    next_input = ni;
                } else {
                    break;
                }
            }
        }

        if result.is_empty() {
            return Err(InternalError::EmptyList(input.span()).into());
        }

        Ok((result, next_input))
    }
}

pub fn separated<'a, P1, P2, Input, Output>(
    parser: P1,
    separator: P2,
) -> impl Parser<'a, Input, Vec<Output>>
where
    Input: ParseStream<'a> + 'a,
    P1: Parser<'a, Input, Output>,
    P2: Parser<'a, Input, Output>,
{
    move |input: Input| {
        let mut result = Vec::new();

        let mut next_input: Option<Input> = Some(input.clone());

        while let Some(ref inp) = next_input.clone() {
            if let Ok((next_item, ni)) = parser.parse(inp.clone()) {
                next_input = ni;
                result.push(next_item);
            } else {
                break;
            }

            if let Some(ref inp) = next_input.clone() {
                if let Ok((next_item, ni)) = separator.parse(inp.clone()) {
                    next_input = ni;
                    result.push(next_item);
                } else {
                    break;
                }
            }
        }

        if result.is_empty() {
            return Err(InternalError::EmptyList(input.span()).into());
        }

        Ok((result, next_input))
    }
}

#[cfg(test)]
mod tests {
    use crate::combinators::literal;
    use crate::parse_stream::StrStream;
    use crate::Parser;

    use super::*;

    #[test]
    fn test_repeat() {
        let input1 = "test,test,test,";
        let input2 = "test,test,test";
        let input3 = ",test,test,test";
        let stream1: StrStream = input1.into();
        let stream2: StrStream = input2.into();
        let stream3: StrStream = input3.into();

        let matcher1 = interleaved(literal("test"), literal(",").void());
        let matcher2 = separated_ignore(literal("test"), literal(",").void());

        assert!(matcher1.parse(stream1.clone()).is_ok());
        assert!(matcher1.parse(stream2.clone()).is_ok());
        assert!(matcher1.parse(stream3.clone()).is_ok());
        assert!(matcher2.parse(stream1.clone()).is_ok());
        assert!(matcher2.parse(stream2.clone()).is_ok());
        assert!(matcher2.parse(stream3.clone()).is_err());
    }
}
