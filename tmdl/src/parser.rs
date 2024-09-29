use crate::lexer::lex;
use crate::token::{Token, TokenStream};
use crate::{CSTNode, InstrNode, InstrTemplateNode, PropertiesNode};
use lpl::combinators::*;
use lpl::{Parser, ParserError, Spanned};

pub fn parse(input: &str) -> Result<Vec<CSTNode>, ParserError> {
    let tokens = lex(input)?;
    let stream = TokenStream::new(&tokens);
    let parser = zero_or_more(parse_top_level());
    let (nodes, _) = parser.parse(stream)?;
    Ok(nodes)
}

fn token<'a>(expected: Token<'a>) -> impl Parser<'a, TokenStream<'a>, ()> {
    move |input: &mut TokenStream<'a>| {
        if let Some((token, span)) = input.next() {
            if token == expected {
                Ok(((), span))
            } else {
                Err(ParserError::UnexpectedToken(span))
            }
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }
}

fn parse_top_level<'a>() -> impl Parser<'a, TokenStream<'a>, CSTNode<'a>> {
    parse_instr_template()
        .or_else(parse_properties())
        .or_else(parse_instr())
}

fn parse_instr_template<'a>() -> impl Parser<'a, TokenStream<'a>, CSTNode<'a>> {
    token(Token::InstrTemplate)
        .and_then(parse_identifier())
        .and_then(parse_parameters())
        .and_then(parse_body())
        .map(|(((_, name), params), body)| {
            CSTNode::InstrTemplate(Box::new(InstrTemplateNode {
                name,
                parameters: params,
                body,
            }))
        })
}

fn parse_properties<'a>() -> impl Parser<'a, TokenStream<'a>, CSTNode<'a>> {
    token(Token::Properties)
        .and_then(token(Token::For))
        .and_then(parse_identifier())
        .and_then(parse_property_body())
        .map(|(((_, _), target), properties)| {
            CSTNode::Properties(Box::new(PropertiesNode { target, properties }))
        })
}

fn parse_instr<'a>() -> impl Parser<'a, TokenStream<'a>, CSTNode<'a>> {
    token(Token::Instr)
        .and_then(parse_identifier())
        .and_then(token(Token::Colon))
        .and_then(parse_identifier())
        .and_then(parse_arguments())
        .map(|((((_, name), _), template), args)| {
            CSTNode::Instr(Box::new(InstrNode {
                name,
                template,
                arguments: args,
            }))
        })
}

fn parse_identifier<'a>() -> impl Parser<'a, TokenStream<'a>, Spanned<&'a str>> {
    todo()
}

fn parse_parameters<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<Spanned<&'a str>>> {
    todo()
}

fn parse_body<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<CSTNode<'a>>> {
    todo()
}

fn parse_property_body<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<CSTNode<'a>>> {
    todo()
}

fn parse_arguments<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<Spanned<&'a str>>> {
    todo()
}
