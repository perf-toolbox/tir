use smallvec::{Array, SmallVec};

use crate::{ParseStream, Parser};

pub trait Flatten {
    type Output;

    fn flatten(self) -> Self::Output;
}

pub trait NotTuple {}

pub fn flat<'a, P, Input, Output1, Output2>(parser: P) -> impl Parser<'a, Input, Output2>
where
    Input: ParseStream<'a> + 'a,
    P: Parser<'a, Input, Output1>,
    Output1: Flatten<Output = Output2>,
{
    move |input: Input| {
        parser
            .parse(input)
            .map(|(result, next_input)| (result.flatten(), next_input))
    }
}

impl NotTuple for () {}
impl NotTuple for bool {}
impl NotTuple for i8 {}
impl NotTuple for i16 {}
impl NotTuple for i32 {}
impl NotTuple for i64 {}
impl NotTuple for &str {}
impl NotTuple for String {}
impl<T> NotTuple for Vec<T> {}
impl<T> NotTuple for Option<T> {}
impl<T> NotTuple for std::sync::Arc<T> {}
impl<A: Array> NotTuple for SmallVec<A> {}

impl<T1, T2, T3> Flatten for (T1, (T2, T3))
where
    T3: NotTuple,
{
    type Output = (T1, T2, T3);

    fn flatten(self) -> Self::Output {
        (self.0, self.1 .0, self.1 .1)
    }
}

impl<T1, T2, T3> Flatten for ((T1, T2), T3)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
{
    type Output = (T1, T2, T3);

    fn flatten(self) -> Self::Output {
        (self.0 .0, self.0 .1, self.1)
    }
}

impl<T1, T2, T3, T4> Flatten for (((T1, T2), T3), T4)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
{
    type Output = (T1, T2, T3, T4);

    fn flatten(self) -> Self::Output {
        (self.0 .0 .0, self.0 .0 .1, self.0 .1, self.1)
    }
}

impl<T1, T2, T3, T4, T5> Flatten for ((((T1, T2), T3), T4), T5)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6> Flatten for (((((T1, T2), T3), T4), T5), T6)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
    T6: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5, T6);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0 .0,
            self.0 .0 .0 .0 .1,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> Flatten for ((((((T1, T2), T3), T4), T5), T6), T7)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
    T6: NotTuple,
    T7: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5, T6, T7);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0 .0 .0,
            self.0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .1,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> Flatten for (((((((T1, T2), T3), T4), T5), T6), T7), T8)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
    T6: NotTuple,
    T7: NotTuple,
    T8: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5, T6, T7, T8);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0 .0 .0 .0,
            self.0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .1,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> Flatten
    for ((((((((T1, T2), T3), T4), T5), T6), T7), T8), T9)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
    T6: NotTuple,
    T7: NotTuple,
    T8: NotTuple,
    T9: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5, T6, T7, T8, T9);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0 .0 .0 .0 .0,
            self.0 .0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .1,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Flatten
    for (((((((((T1, T2), T3), T4), T5), T6), T7), T8), T9), T10)
where
    T1: NotTuple,
    T2: NotTuple,
    T3: NotTuple,
    T4: NotTuple,
    T5: NotTuple,
    T6: NotTuple,
    T7: NotTuple,
    T8: NotTuple,
    T9: NotTuple,
    T10: NotTuple,
{
    type Output = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

    fn flatten(self) -> Self::Output {
        (
            self.0 .0 .0 .0 .0 .0 .0 .0 .0,
            self.0 .0 .0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .0 .1,
            self.0 .0 .0 .0 .1,
            self.0 .0 .0 .1,
            self.0 .0 .1,
            self.0 .1,
            self.1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Flatten;

    #[test]
    fn flatten_simple_tuples() {
        assert_eq!(((1, 2), 3).flatten(), (1, 2, 3))
    }
}
