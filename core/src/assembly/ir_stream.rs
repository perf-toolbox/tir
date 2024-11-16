use std::{
    cell::RefCell,
    ops::{Range, RangeBounds},
    sync::Arc,
};

use crate::{ContextRef, RegionRef, Type};
use lpl::{ParseStream, Span};

#[derive(Debug, Clone)]
pub struct IRStrStream<'a> {
    string: &'a str,
    filename: Arc<String>,
    offset: usize,
    state: Arc<ParserState>,
}

#[derive(Debug)]
pub struct ParserStateImpl {
    context: ContextRef,
    deferred_type_list: Vec<Type>,
    deferred_arg_names: Vec<String>,
    cur_region: Vec<RegionRef>,
}

#[derive(Debug)]
pub struct ParserState(RefCell<ParserStateImpl>);

impl<'a> IRStrStream<'a> {
    pub fn new(string: &'a str, filename: &'a str, context: ContextRef) -> Self {
        let filename = Arc::new(filename.to_string());
        let state = ParserState::new(context);
        Self {
            string,
            filename,
            offset: 0,
            state,
        }
    }
}

impl<'a> ParseStream<'a> for IRStrStream<'a> {
    type Slice = &'a str;
    type Extra = Arc<ParserState>;
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
                        filename: self.filename.clone(),
                        offset,
                        state: self.state.clone(),
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
        Span::unbound(Some(self.filename.clone()), self.offset)
    }

    fn set_extra(&mut self, extra: Self::Extra) {
        self.state = extra;
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        Some(&self.state)
    }

    fn peek(&self) -> Option<Self::Item> {
        self.string.chars().next()
    }

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.string.chars().nth(n)
    }
}

unsafe impl Send for ParserState {}
unsafe impl Sync for ParserState {}

impl ParserState {
    pub fn new(context: ContextRef) -> Arc<Self> {
        Arc::new(ParserState(RefCell::new(ParserStateImpl {
            context,
            deferred_type_list: vec![],
            deferred_arg_names: vec![],
            cur_region: vec![],
        })))
    }

    pub fn context(&self) -> ContextRef {
        self.0.borrow().context.clone()
    }

    pub fn push_region(&self, region: RegionRef) {
        self.0.borrow_mut().cur_region.push(region);
    }

    pub fn get_region(&self) -> RegionRef {
        self.0.borrow().cur_region.last().cloned().unwrap()
    }

    pub fn pop_region(&self) {
        self.0.borrow_mut().cur_region.pop();
    }

    pub fn take_deferred_types(&self) -> Vec<Type> {
        std::mem::take(&mut self.0.borrow_mut().deferred_type_list)
    }

    pub fn set_deferred_types(&self, types: Vec<Type>) {
        self.0.borrow_mut().deferred_type_list = types;
    }

    pub fn take_deferred_names(&self) -> Vec<String> {
        std::mem::take(&mut self.0.borrow_mut().deferred_arg_names)
    }

    pub fn set_deferred_names(&self, names: Vec<String>) {
        self.0.borrow_mut().deferred_arg_names = names;
    }
}
