use std::ops::{Range, RangeBounds};

use crate::ContextRef;
use lpl::{ParseStream, Span};

#[derive(Debug, Clone)]
pub struct IRStrStream<'a> {
    string: &'a str,
    offset: usize,
    context: ContextRef,
}

impl<'a> IRStrStream<'a> {
    pub fn new(string: &'a str, context: ContextRef) -> Self {
        Self {
            string,
            offset: 0,
            context,
        }
    }
}

impl<'a> ParseStream<'a> for IRStrStream<'a> {
    type Slice = &'a str;
    type Extra = ContextRef;
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
                    Some(Self {
                        string,
                        offset,
                        context: self.context.clone(),
                    })
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

    fn span(&self) -> Span {
        Span::unbound(None, self.offset)
    }

    fn set_extra(&mut self, extra: Self::Extra) {
        self.context = extra;
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        Some(&self.context)
    }

    fn peek(&self) -> Option<Self::Item> {
        self.string.chars().next()
    }

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.string.chars().nth(n)
    }
}
