use std::{
    fmt,
    ops::{Range, RangeBounds},
};

use crate::{
    syntax::{GreenElement, SyntaxLike},
    Span,
};

pub trait ParseStream<'a>: Clone + fmt::Debug {
    type Slice;
    type Extra;
    type Item;

    fn nth(&self, n: usize) -> Option<Self::Item>;

    fn get<R>(&self, range: R) -> Option<Self::Slice>
    where
        R: RangeBounds<usize>;

    fn slice<R>(&self, range: R) -> Option<Self>
    where
        Self: Sized,
        R: RangeBounds<usize>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn span(&self) -> Span;

    fn set_extra(&mut self, extra: Self::Extra);
    fn get_extra(&self) -> Option<&Self::Extra>;

    fn peek(&self) -> Option<Self::Item>;

    fn is_string_like(&self) -> bool {
        false
    }

    fn chars(&self) -> std::str::Chars<'_> {
        unimplemented!()
    }

    fn substr(&self, _range: Range<usize>) -> Option<&'a str> {
        unimplemented!()
    }

    fn starts_with(&self, _prefix: &str) -> bool {
        unimplemented!()
    }
}

pub trait TokenLike: Clone + fmt::Debug {
    fn span(&self) -> Span;
    fn is_trivia(&self) -> bool;
}

#[derive(Clone)]
pub struct StrStream<'a> {
    string: &'a str,
    offset: usize,
}

#[derive(Clone)]
pub struct TokenStream<'a, SK: SyntaxLike> {
    tokens: &'a [GreenElement<SK>],
    offset: usize,
}

impl<'a> ParseStream<'a> for StrStream<'a> {
    type Slice = &'a str;
    type Extra = ();
    type Item = char;

    fn get<R>(&self, range: R) -> Option<Self::Slice>
    where
        R: RangeBounds<usize>,
    {
        self.string
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
    }

    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize>,
    {
        let offset = match range.start_bound() {
            std::ops::Bound::Included(bound) => self.offset + bound,
            std::ops::Bound::Excluded(bound) => self.offset + bound + 1,
            std::ops::Bound::Unbounded => self.offset,
        };
        self.string
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
            .and_then(|string| {
                if string.is_empty() {
                    None
                } else {
                    Some(Self { string, offset })
                }
            })
    }

    fn len(&self) -> usize {
        self.string.len()
    }

    fn is_string_like(&self) -> bool {
        true
    }

    fn chars(&self) -> std::str::Chars<'_> {
        self.string.chars()
    }

    fn substr(&self, range: Range<usize>) -> Option<&'a str> {
        self.string.get(range)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.string.starts_with(prefix)
    }

    fn span(&self) -> Span {
        Span::unbound(None, self.offset)
    }

    fn set_extra(&mut self, _extra: Self::Extra) {
        unimplemented!("default string stream does not support attached extra info")
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        unimplemented!("default string stream does not support attached extra info")
    }

    fn peek(&self) -> Option<Self::Item> {
        self.string.chars().next()
    }

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.string.chars().nth(n)
    }
}

impl<'a> From<&'a str> for StrStream<'a> {
    fn from(string: &'a str) -> Self {
        StrStream { string, offset: 0 }
    }
}

impl<'a> fmt::Debug for StrStream<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:30}", self.string)
    }
}

impl<'a, SK: SyntaxLike> TokenStream<'a, SK> {
    pub fn new(tokens: &'a [GreenElement<SK>]) -> Self {
        Self { tokens, offset: 0 }
    }
}

impl<'a, SK: SyntaxLike> ParseStream<'a> for TokenStream<'a, SK> {
    type Slice = &'a [GreenElement<SK>];

    type Extra = ();

    type Item = GreenElement<SK>;

    fn get<R>(&self, range: R) -> Option<Self::Slice>
    where
        R: RangeBounds<usize>,
    {
        self.tokens
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
    }

    fn slice<R>(&self, range: R) -> Option<Self>
    where
        Self: Sized,
        R: RangeBounds<usize>,
    {
        let offset = match range.start_bound() {
            std::ops::Bound::Included(bound) => self.offset + bound,
            std::ops::Bound::Excluded(bound) => self.offset + bound + 1,
            std::ops::Bound::Unbounded => self.offset,
        };
        self.tokens
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
            .map(|tokens| Self { tokens, offset })
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn span(&self) -> Span {
        self.tokens
            .first()
            .map(|t| t.as_token().span())
            .unwrap_or_default()
    }

    fn set_extra(&mut self, _extra: Self::Extra) {
        todo!()
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        todo!()
    }

    fn peek(&self) -> Option<Self::Item> {
        self.tokens.first().cloned()
    }

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.tokens.get(n).cloned()
    }
}

impl<'a, SK: SyntaxLike> fmt::Debug for TokenStream<'a, SK> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = self.tokens.len().min(5);
        write!(f, "offset: {} - {:?}", self.offset, &self.tokens[..end])
    }
}
