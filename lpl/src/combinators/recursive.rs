use std::{
    cell::OnceCell,
    marker::PhantomData,
    rc::{self, Rc},
};

use crate::{ParseStream, Parser};

#[derive(Clone)]
pub struct Deferred<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    inner: OnceCell<P>,
    _a: PhantomData<Input>,
    _b: PhantomData<Output>,
    _c: PhantomData<&'a ()>,
}

impl<'a, Input, Output, P> Deferred<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    pub fn declare() -> Self {
        Self {
            inner: OnceCell::new(),
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        }
    }

    pub fn define(&mut self, parser: P) {
        if self.inner.set(parser).is_err() {
            panic!("Parser can only be defined once")
        }
    }
}

impl<'a, Input, Output, P> Parser<'a, Input, Output> for Deferred<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    fn parse(&self, input: Input) -> crate::ParseResult<Input, Output> {
        self.inner.get().unwrap().parse(input)
    }
}

#[derive(Clone)]
enum RecursiveInner<T: ?Sized> {
    Owned(Rc<T>),
    Unowned(rc::Weak<T>),
}

pub struct Recursive<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output> + ?Sized,
{
    inner: RecursiveInner<P>,
    _a: PhantomData<Input>,
    _b: PhantomData<Output>,
    _c: PhantomData<&'a ()>,
}

impl<'a, Input, Output, P> Recursive<'a, Input, Output, Deferred<'a, Input, Output, P>>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output>,
{
    pub fn declare() -> Self {
        Self {
            inner: RecursiveInner::Owned(Rc::new(Deferred {
                inner: OnceCell::new(),
                _a: PhantomData,
                _b: PhantomData,
                _c: PhantomData,
            })),
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        }
    }

    pub fn define(&mut self, parser: P) {
        match &self.inner {
            RecursiveInner::Owned(inner) => {
                if inner.inner.set(parser).is_err() {
                    panic!("Parser can only be defined once")
                }
            }
            _ => unreachable!("Define can only be called on owned parsers"),
        };
    }
}

impl<'a, Input, Output, P> Parser<'a, Input, Output> for Recursive<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output> + ?Sized,
{
    fn parse(&self, input: Input) -> crate::ParseResult<Input, Output> {
        match &self.inner {
            RecursiveInner::Owned(p) => p.parse(input),
            RecursiveInner::Unowned(p) => p
                .upgrade()
                .expect("Parser used before defining")
                .parse(input),
        }
    }
}

impl<'a, Input, Output, P> Clone for Recursive<'a, Input, Output, P>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output> + ?Sized + 'a,
{
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                RecursiveInner::Owned(p) => RecursiveInner::Owned(p.clone()),
                RecursiveInner::Unowned(p) => RecursiveInner::Unowned(p.clone()),
            },
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        }
    }
}

pub fn recursive<'a, 'b, Input, Output, F, P1>(f: F) -> Recursive<'a, Input, Output, P1>
where
    Input: ParseStream<'a> + 'a,
    Output: 'a,
    P1: Parser<'a, Input, Output> + 'a,
    F: FnOnce(Recursive<'a, Input, Output, dyn Parser<'a, Input, Output> + 'a>) -> P1,
{
    let rc = Rc::new_cyclic(|rc| {
        let rc: rc::Weak<dyn Parser<'a, Input, Output>> = rc.clone() as _;
        let parser = Recursive {
            inner: RecursiveInner::Unowned(rc.clone()),
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        };

        f(parser)
    });

    Recursive {
        inner: RecursiveInner::Owned(rc),
        _a: PhantomData,
        _b: PhantomData,
        _c: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        combinators::{literal, Recursive},
        Parser, StrStream,
    };

    #[test]
    fn deferred_parser() {
        let mut deferred = Recursive::declare();

        deferred.define(literal("hello"));

        let stream: StrStream = "hello".into();

        let result = deferred.parse(stream);
        assert!(result.is_ok());
    }
}
