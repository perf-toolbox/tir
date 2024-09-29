use crate::combinators::pred;
use crate::ParseStream;
use crate::Parser;

use super::one_or_more;
use super::text::any_char;
use super::zero_or_more;

pub fn any_whitespace<'a, Input>() -> impl Parser<'a, Input, char>
where
    Input: ParseStream<'a> + 'a,
{
    pred(any_char, |c| c.is_whitespace())
}

pub fn any_whitespace0<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: ParseStream<'a> + 'a,
{
    zero_or_more(any_whitespace()).map(|_| ())
}

pub fn any_whitespace1<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: ParseStream<'a> + 'a,
{
    one_or_more(any_whitespace()).map(|_| ())
}

pub fn space<'a, Input>() -> impl Parser<'a, Input, char>
where
    Input: ParseStream<'a> + 'a,
{
    pred(any_char, |c| *c == ' ' || *c == '\t')
}

pub fn space0<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: ParseStream<'a> + 'a,
{
    zero_or_more(space()).map(|_| ())
}

pub fn space1<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: ParseStream<'a> + 'a,
{
    one_or_more(space()).map(|_| ())
}

pub fn spaced<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output> + 'a,
    Output: 'a,
{
    space1()
        .and_then(parser)
        .and_then(space1())
        .map(|((_, d), _)| d)
}

#[cfg(test)]
mod tests {
    use crate::combinators::space1;
    use crate::parse_stream::StrStream;
    use crate::Parser;

    use super::*;

    #[test]
    fn match_whitespace() {
        let input = "  \tSpace";
        let with_space: StrStream = input.into();
        let input2 = "NoSpaceString";
        let no_space: StrStream = input2.into();

        let space_matcher = space1();
        let space0_matcher = space0();
        let anyspace_matcher = any_whitespace1();
        let anyspace0_matcher = any_whitespace0();

        assert!(space_matcher.parse(with_space.clone()).is_ok());
        assert!(space_matcher.parse(no_space.clone()).is_err());
        assert!(space0_matcher.parse(with_space.clone()).is_ok());
        assert!(space0_matcher.parse(no_space.clone()).is_ok());
        assert!(anyspace_matcher.parse(with_space.clone()).is_ok());
        assert!(anyspace_matcher.parse(no_space.clone()).is_err());
        assert!(anyspace0_matcher.parse(with_space.clone()).is_ok());
        assert!(anyspace0_matcher.parse(no_space.clone()).is_ok());
    }
}
