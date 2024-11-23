use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{BoxedParser, ParseResult, ParseStream, Parser};

static DEPTH_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct LabelledParser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    label: &'static str,
    parser: BoxedParser<'a, Input, Output>,
}

impl<'a, Input, Output> LabelledParser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    pub fn new(label: &'static str, parser: BoxedParser<'a, Input, Output>) -> Self {
        LabelledParser { label, parser }
    }
}

impl<'a, Input, Output> Parser<'a, Input, Output> for LabelledParser<'a, Input, Output>
where
    Input: ParseStream<'a> + 'a,
{
    fn parse(&self, input: Input) -> ParseResult<Input, Output> {
        let print_info = std::env::var("LPL_TRACE").is_ok();
        let offset = DEPTH_COUNTER.fetch_add(1, Ordering::SeqCst);

        if print_info {
            eprintln!();
            for _ in 0..offset {
                eprint!(" |");
            }
            eprint!(" |- {} -> {:?} - '{:?}'", self.label, input.span(), input);
        }

        let result = self.parser.parse(input);

        if print_info {
            eprintln!();
            for _ in 0..offset {
                eprint!(" |");
            }

            if result.is_ok() {
                eprint!(" |-> ok");
            } else {
                eprintln!(" |-> err {:?}", result.as_ref().err().unwrap());
            }
        }

        DEPTH_COUNTER.fetch_sub(1, Ordering::SeqCst);

        result
    }
}
