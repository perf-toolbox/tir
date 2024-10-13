pub mod combinators;
pub mod syntax;

mod err;
mod parse_stream;

pub use err::*;
pub use parse_stream::*;

use core::fmt;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct Span {
    filename: Option<Arc<String>>,
    offset_start: usize,
    offset_end: usize,
}

impl Span {
    pub fn new(filename: Option<Arc<String>>, offset_start: usize, offset_end: usize) -> Self {
        Self {
            filename,
            offset_start,
            offset_end,
        }
    }

    pub fn empty() -> Self {
        Self::new(None, 0, 0)
    }

    pub fn unbound(filename: Option<Arc<String>>, offset_start: usize) -> Self {
        Self::new(filename, offset_start, usize::MAX)
    }

    pub fn get_filename(&self) -> Option<&str> {
        match &self.filename {
            Some(filename) => Some(filename.as_ref()),
            None => None,
        }
    }

    pub fn clone_filename(&self) -> Option<Arc<String>> {
        self.filename.clone()
    }

    pub fn get_offset_start(&self) -> usize {
        self.offset_start
    }

    pub fn get_offset_end(&self) -> usize {
        self.offset_end
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref name) = self.filename {
            fmt::Debug::fmt(name, f)?;
        } else {
            write!(f, "<unknown>")?;
        }
        write!(f, "@")?;
        fmt::Debug::fmt(&self.offset_start, f)?;
        write!(f, "..")?;
        fmt::Debug::fmt(&self.offset_end, f)
    }
}

pub type Spanned<Type> = (Type, Span);

pub type ParseResult<Input, Output> = Result<(Output, Option<Input>), ParserError>;

pub trait Parser<'a, Input: ParseStream<'a> + 'a, Output> {
    fn parse(&self, input: Input) -> ParseResult<Input, Output>;

    fn spanned(self) -> BoxedParser<'a, Input, Spanned<Output>>
    where
        Self: Sized + 'a,
        Output: 'a,
    {
        BoxedParser::new(combinators::spanned(self))
    }

    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, Input, NewOutput>
    where
        Self: Sized + 'a,
        Output: 'a,
        NewOutput: 'a,
        F: Fn(Output) -> NewOutput + 'a,
    {
        BoxedParser::new(combinators::map(self, map_fn))
    }

    fn or_else<P2>(self, parser2: P2) -> BoxedParser<'a, Input, Output>
    where
        P2: Parser<'a, Input, Output> + 'a,
        Self: Sized + 'a,
        Output: 'a,
    {
        BoxedParser::new(combinators::or_else(self, parser2))
    }

    fn and_then<P2, Output2>(self, parser2: P2) -> BoxedParser<'a, Input, (Output, Output2)>
    where
        Output: 'a,
        Output2: 'a,
        Self: Sized + 'a,
        P2: Parser<'a, Input, Output2> + 'a,
    {
        BoxedParser::new(combinators::and_then(self, parser2))
    }

    fn void(self) -> BoxedParser<'a, Input, ()>
    where
        Self: Sized + 'a,
        Output: 'a,
    {
        BoxedParser::new(combinators::map(self, |_| ()))
    }
}

impl<'a, F, Input, Output> Parser<'a, Input, Output> for F
where
    F: Fn(Input) -> ParseResult<Input, Output>,
    Input: ParseStream<'a> + 'a,
{
    fn parse(&self, input: Input) -> ParseResult<Input, Output> {
        self(input)
    }
}

pub struct BoxedParser<'a, Input, Output>
where
    Input: ParseStream<'a>,
{
    parser: Box<dyn Parser<'a, Input, Output> + 'a>,
}

impl<'a, Input: ParseStream<'a>, Output> BoxedParser<'a, Input, Output> {
    pub fn new<P>(parser: P) -> Self
    where
        P: Parser<'a, Input, Output> + 'a,
    {
        BoxedParser {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Input: ParseStream<'a>, Output> Parser<'a, Input, Output>
    for BoxedParser<'a, Input, Output>
{
    fn parse(&self, input: Input) -> ParseResult<Input, Output> {
        self.parser.parse(input)
    }
}
