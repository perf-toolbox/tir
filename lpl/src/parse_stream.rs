use std::ops::{Range, RangeBounds};

use crate::Span;

pub trait ParseStream<'a>: Clone {
    type Slice;

    fn get(&self, range: Range<usize>) -> Option<Self::Slice>;
    fn slice(&self, range: Range<usize>) -> Option<Self>
    where
        Self: Sized;
    fn len(&self) -> usize;

    fn span(&self) -> Span;

    fn is_string_like(&self) -> bool {
        false
    }

    fn chars(&self) -> std::str::Chars<'_> {
        unimplemented!()
    }

    fn substr(&self, _range: Range<usize>) -> Option<&'a str> {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct StrStream<'a> {
    string: &'a str,
    offset: usize,
}

impl<'a> ParseStream<'a> for StrStream<'a> {
    type Slice = &'a str;

    fn get(&self, range: Range<usize>) -> Option<Self::Slice> {
        self.string.get(range)
    }

    fn slice(&self, range: Range<usize>) -> Option<Self> {
        let offset = match range.start_bound() {
            std::ops::Bound::Included(bound) => self.offset + bound,
            std::ops::Bound::Excluded(bound) => self.offset + bound + 1,
            std::ops::Bound::Unbounded => self.offset,
        };
        self.string.get(range).map(|string| Self { string, offset })
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

    fn span(&self) -> Span {
        Span::unbound(None, self.offset)
    }
}

impl<'a> From<&'a str> for StrStream<'a> {
    fn from(string: &'a str) -> Self {
        StrStream { string, offset: 0 }
    }
}
