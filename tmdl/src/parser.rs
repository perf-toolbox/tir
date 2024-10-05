use lpl::syntax::GreenNodeData;
use lpl::{combinators::*, ParserError, Span};
use lpl::{
    syntax::{GreenElement, GreenNode, NodeOrToken},
    ParseStream, Parser,
};

use crate::{SyntaxKind, TokenStream};

pub fn parse(tokens: &[GreenElement<SyntaxKind>]) -> GreenNode<SyntaxKind> {
    let stream = TokenStream::new(tokens);

    let top_level_decl = parse_instr_template_decl();
    let parser = zero_or_more(top_level_decl.map(NodeOrToken::Node).or_else(catch_all()));

    let result = parser.parse(stream).unwrap();

    GreenNodeData::new(SyntaxKind::TranslationUnit, result.0, Span::empty())
}

fn catch_all<'a>() -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    move |tokens: TokenStream<'a>| {
        if tokens.len() > 0 {
            return Ok((tokens.nth(0).unwrap(), tokens.slice(1..)));
        }
        Err(ParserError::new("end of file", Span::empty()))
    }
}

fn eat_until<'a>(
    kind: SyntaxKind,
) -> impl Parser<'a, TokenStream<'a>, Vec<GreenElement<SyntaxKind>>> {
    move |tokens: TokenStream<'a>| {
        let mut alien_tokens = vec![];
        for i in 0..tokens.len() {
            if let Some(element) = tokens.nth(i) {
                if let NodeOrToken::Token(token) = &element {
                    if token.kind() == kind {
                        return Ok((alien_tokens, tokens.slice(i..)));
                    } else {
                        alien_tokens.push(element);
                    }
                }
            }
        }

        Err(ParserError::new(
            format!("Expected `{:?}` not found", kind),
            tokens.span(),
        ))
    }
}

fn eat_until_one_of<'a>(
    kinds: &'static [SyntaxKind],
) -> impl Parser<'a, TokenStream<'a>, Vec<GreenElement<SyntaxKind>>> {
    move |tokens: TokenStream<'a>| {
        let mut alien_tokens = vec![];
        for i in 0..tokens.len() {
            if let Some(element) = tokens.nth(i) {
                if let NodeOrToken::Token(token) = &element {
                    if kinds.contains(&token.kind()) {
                        return Ok((alien_tokens, tokens.slice(i..)));
                    } else {
                        alien_tokens.push(element);
                    }
                }
            }
        }

        Err(ParserError::new("Expected tokens not found", tokens.span()))
    }
}

fn token<'a>(kind: SyntaxKind) -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    move |tokens: TokenStream<'a>| {
        if let Some(element) = tokens.nth(0) {
            if let NodeOrToken::Token(token) = &element {
                if token.kind() == kind {
                    return Ok((element, tokens.slice(1..)));
                }
            }
        }

        Err(ParserError::new(
            format!("Expected `{:?}` not found", kind),
            tokens.span(),
        ))
    }
}

fn token_of<'a>(
    kinds: &'static [SyntaxKind],
) -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    move |tokens: TokenStream<'a>| {
        if let Some(element) = tokens.nth(0) {
            if let NodeOrToken::Token(token) = &element {
                if kinds.contains(&token.kind()) {
                    return Ok((element, tokens.slice(1..)));
                }
            }
        }

        Err(ParserError::new("Expected tokens not found", tokens.span()))
    }
}

fn parse_instr_template_decl<'a>() -> impl Parser<'a, TokenStream<'a>, GreenNode<SyntaxKind>> {
    token(SyntaxKind::InstrTemplateKw)
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::Identifier))
        .map(move |((kw, aliens), name)| (kw, aliens, name))
        .and_then(eat_until(SyntaxKind::LeftAngle))
        .map(|((kw, aliens1, name), aliens2)| (kw, aliens1, name, aliens2))
        .and_then(parse_instr_template_parameters())
        .map(|((kw, aliens1, name, aliens2), params)| (kw, aliens1, name, aliens2, params))
        .and_then(eat_until(SyntaxKind::LeftBrace))
        .map(|((kw, aliens1, name, aliens2, params), aliens3)| {
            (kw, aliens1, name, aliens2, params, aliens3)
        })
        .and_then(parse_struct_body())
        .map(|((kw, aliens1, name, aliens2, params, aliens3), body)| {
            (kw, aliens1, name, aliens2, params, aliens3, body)
        })
        .map(|(kw, aliens1, name, aliens2, params, aliens3, body)| {
            let mut elements = vec![];
            let kw_span = kw.as_token().span();
            elements.push(kw);
            elements.extend(aliens1);
            let name_span = name.as_token().span();
            elements.push(GreenElement::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateName,
                vec![name],
                name_span,
            )));
            elements.extend(aliens2);
            elements.push(NodeOrToken::Node(params));
            elements.extend(aliens3);
            elements.push(NodeOrToken::Node(body));

            GreenNodeData::new(SyntaxKind::InstrTemplateDecl, elements, kw_span)
        })
}

fn parse_type<'a>() -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    token(SyntaxKind::Identifier)
        .and_then(optional(
            token(SyntaxKind::LeftAngle)
                .and_then(token_of(&[
                    SyntaxKind::IntegerLiteral,
                    SyntaxKind::StringLiteral,
                    SyntaxKind::BitLiteral,
                ]))
                .and_then(token(SyntaxKind::RightAngle))
                .map(|((angle_start, lit), angle_end)| {
                    let span = angle_start.as_token().span();
                    let elements = vec![angle_start, lit, angle_end];
                    NodeOrToken::Node(GreenNodeData::new(SyntaxKind::TypeParams, elements, span))
                }),
        ))
        .map(|(ident, params)| {
            let span = ident.as_token().span();
            let mut elements = vec![ident];
            if let Some(params) = params {
                elements.push(params)
            }
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::Type, elements, span))
        })
}

fn parse_single_template_parameter<'a>(
) -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    eat_until(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until(SyntaxKind::Colon))
        .and_then(token(SyntaxKind::Colon))
        .map(|(((aliens0, name), aliens1), colon)| (aliens0, name, aliens1, colon))
        .and_then(eat_until(SyntaxKind::Identifier))
        .map(|((aliens0, name, aliens1, colon), aliens2)| (aliens0, name, aliens1, colon, aliens2))
        .and_then(parse_type())
        .map(|((aliens0, name, aliens1, colon, aliens2), ty)| {
            (aliens0, name, aliens1, colon, aliens2, ty)
        })
        .and_then(eat_until_one_of(&[
            SyntaxKind::Comma,
            SyntaxKind::RightAngle,
        ]))
        .map(|((aliens0, name, aliens1, colon, aliens2, ty), aliens3)| {
            (aliens0, name, aliens1, colon, aliens2, ty, aliens3)
        })
        .map(|(aliens0, name, aliens1, colon, aliens2, ty, aliens3)| {
            let mut elements = vec![];
            let span = name.as_token().span();
            elements.extend(aliens0);
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::StructFieldName,
                vec![name],
                span.clone(),
            )));
            elements.extend(aliens1);
            elements.push(colon);
            elements.extend(aliens2);
            elements.push(ty);
            elements.extend(aliens3);
            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateSingleParam,
                elements,
                span,
            ))
        })
}

fn parse_instr_template_parameters<'a>() -> impl Parser<'a, TokenStream<'a>, GreenNode<SyntaxKind>>
{
    token(SyntaxKind::LeftAngle)
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(separated(
            parse_single_template_parameter(),
            token(SyntaxKind::Comma).map(|_| ()),
        ))
        .map(|((angle, aliens), params)| (angle, aliens, params))
        .and_then(eat_until(SyntaxKind::RightAngle))
        .map(|((angle, aliens1, params), aliens2)| (angle, aliens1, params, aliens2))
        .and_then(token(SyntaxKind::RightAngle))
        .map(|((angle_start, aliens1, params, aliens2), angle_end)| {
            (angle_start, aliens1, params, aliens2, angle_end)
        })
        .map(|(angle_start, aliens1, params, aliens2, angle_end)| {
            let mut elements = vec![];
            let span = angle_start.as_token().span();
            elements.push(angle_start);
            elements.extend(aliens1);
            elements.extend(params);
            elements.extend(aliens2);
            elements.push(angle_end);
            GreenNodeData::new(SyntaxKind::InstrTemplateParams, elements, span)
        })
}

fn parse_struct_field<'a>() -> impl Parser<'a, TokenStream<'a>, GreenElement<SyntaxKind>> {
    eat_until(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until(SyntaxKind::Colon))
        .and_then(token(SyntaxKind::Colon))
        .map(|(((aliens0, name), aliens1), colon)| (aliens0, name, aliens1, colon))
        .and_then(eat_until(SyntaxKind::Identifier))
        .map(|((aliens0, name, aliens1, colon), aliens2)| (aliens0, name, aliens1, colon, aliens2))
        .and_then(parse_type())
        .map(|((aliens0, name, aliens1, colon, aliens2), ty)| {
            (aliens0, name, aliens1, colon, aliens2, ty)
        })
        .and_then(eat_until_one_of(&[
            SyntaxKind::Comma,
            SyntaxKind::RightBrace,
        ]))
        .map(|((aliens0, name, aliens1, colon, aliens2, ty), aliens3)| {
            (aliens0, name, aliens1, colon, aliens2, ty, aliens3)
        })
        .map(|(aliens0, name, aliens1, colon, aliens2, ty, aliens3)| {
            let span = name.as_token().span();
            let mut elements = vec![];
            elements.extend(aliens0);
            elements.push(name);
            elements.extend(aliens1);
            elements.push(colon);
            elements.extend(aliens2);
            elements.push(ty);
            elements.extend(aliens3);

            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::StructField, elements, span))
        })
}

fn parse_struct_body<'a>() -> impl Parser<'a, TokenStream<'a>, GreenNode<SyntaxKind>> {
    token(SyntaxKind::LeftBrace)
        .and_then(optional(separated(
            parse_struct_field(),
            token(SyntaxKind::Comma).map(|_| ()),
        )))
        .and_then(eat_until(SyntaxKind::RightBrace))
        .and_then(token(SyntaxKind::RightBrace))
        .map(|(((left_brace, fields), aliens), right_brace)| {
            let span = left_brace.as_token().span();
            let mut elements = vec![];
            elements.push(left_brace);
            if let Some(fields) = fields {
                elements.extend(fields);
            }
            elements.extend(aliens);
            elements.push(right_brace);

            GreenNodeData::new(SyntaxKind::StructBody, elements, span)
        })
}
