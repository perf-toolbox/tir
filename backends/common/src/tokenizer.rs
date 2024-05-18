use std::ops::Range;
use winnow::ascii::{alpha1, alphanumeric0, space0, space1};
use winnow::combinator::{alt, delimited, separated, terminated};
use winnow::token::take_while;
use winnow::{
    ascii::alphanumeric1,
    combinator::{opt, preceded},
    // stream::Range,
    PResult,
};
use winnow::{Located, Parser};

#[derive(Debug, Clone)]
pub enum AsmOperand<'a> {
    Ident(&'a str),
}

#[derive(Debug, Clone)]
pub enum AsmToken<'a> {
    Instruction(&'a str),
    Operand(AsmOperand<'a>, Range<usize>),
    Section(&'a str),
    Label(&'a str),
}

pub type Spanned<'a> = (AsmToken<'a>, Range<usize>);

/// Shortcut for well-known sections like `.text`. These do not need a `.section` prefix.
fn known_section<'a>(input: &mut Located<&'a str>) -> PResult<&'a str> {
    alt((".text", ".data", ".rodata", ".bss")).parse_next(input)
}

fn section<'a>(input: &mut Located<&'a str>) -> PResult<Spanned<'a>> {
    alt((
        known_section,
        preceded((".section", space1), (opt("."), alphanumeric1).recognize()),
    ))
    .map(AsmToken::Section)
    .with_span()
    .parse_next(input)
}

fn label<'a>(input: &mut Located<&'a str>) -> PResult<Spanned<'a>> {
    terminated((alpha1, alphanumeric0).recognize(), ':')
        .map(AsmToken::Label)
        .with_span()
        .parse_next(input)
}

fn operand<'a>(input: &mut Located<&'a str>) -> PResult<(AsmOperand<'a>, Range<usize>)> {
  (alpha1, alphanumeric0).recognize().map(AsmOperand::Ident).with_span().parse_next(input)
}

fn instruction<'a>(input: &mut Located<&'a str>) -> PResult<Spanned<'a>> {
  (alpha1, take_while(0.., |c: char| c.is_alphanumeric() || c == '.' || c == '_')).recognize().map(AsmToken::Instruction).with_span().parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::{Located, Parser};

    use super::{label, section, AsmToken};

    #[test]
    fn sections() {
        let (res, _) = section
            .parse(Located::new(".section .text"))
            .expect("section");
        match res {
            AsmToken::Section(name) => assert_eq!(name, ".text"),
            _ => panic!("Not a section"),
        };

        let (res, _) = section.parse(Located::new(".text")).expect("section");
        match res {
            AsmToken::Section(name) => assert_eq!(name, ".text"),
            _ => panic!("Not a section"),
        };
    }

    #[test]
    fn labels() {
        let (res, _) = label
            .parse(Located::new("foo:"))
            .expect("label");
        match res {
            AsmToken::Label(name) => assert_eq!(name, "foo"),
            _ => panic!("Not a section"),
        };
        assert!(label.parse(Located::new("1bar:")).is_err());
    }
}
