use lpl::syntax::GreenNodeData;
use lpl::{combinators::*, ParserError, Span};
use lpl::{
    syntax::{GreenElement, NodeOrToken},
    ParseStream, Parser,
};

use crate::{ImmElement, ImmNode, SyntaxKind, TokenStream};

pub fn parse(tokens: &[ImmElement]) -> ImmNode {
    let stream = TokenStream::new(tokens);

    let top_level_decl = parse_instr_template_decl()
        .or_else(parse_instr_decl())
        .or_else(parse_encoding())
        .or_else(parse_asm());
    let parser = zero_or_more(top_level_decl.map(NodeOrToken::Node).or_else(catch_all()));

    let result = parser.parse(stream).unwrap();

    GreenNodeData::new(SyntaxKind::TranslationUnit, result.0, Span::empty())
}

fn catch_all<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    move |tokens: TokenStream<'a>| {
        if tokens.len() > 0 {
            return Ok((tokens.nth(0).unwrap(), tokens.slice(1..)));
        }
        Err(ParserError::new("end of file", Span::empty()))
    }
}

fn eat_until<'a>(kind: SyntaxKind) -> impl Parser<'a, TokenStream<'a>, Vec<ImmElement>> {
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

fn eat_whitespace<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<ImmElement>> {
    move |tokens: TokenStream<'a>| {
        let mut alien_tokens = vec![];
        for i in 0..tokens.len() {
            if let Some(element) = tokens.nth(i) {
                if let NodeOrToken::Token(token) = &element {
                    if token.kind() != SyntaxKind::Whitespace {
                        return Ok((alien_tokens, tokens.slice(i..)));
                    } else {
                        alien_tokens.push(element);
                    }
                }
            }
        }

        Ok((alien_tokens, None))
    }
}

fn eat_until_one_of<'a>(
    kinds: &'static [SyntaxKind],
) -> impl Parser<'a, TokenStream<'a>, Vec<ImmElement>> {
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

fn token<'a>(kind: SyntaxKind) -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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

fn token_of<'a>(kinds: &'static [SyntaxKind]) -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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

fn parse_instr_template_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
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

fn parse_type<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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
                    let lit_span = lit.as_token().span();
                    let lit_expr = NodeOrToken::Node(GreenNodeData::new(SyntaxKind::LiteralExpr, vec![lit], lit_span));
                    let elements = vec![angle_start, lit_expr, angle_end];
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

fn parse_single_template_parameter<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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

fn parse_instr_template_parameters<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
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

fn parse_struct_field<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    eat_until_one_of(&[SyntaxKind::Identifier, SyntaxKind::RightBrace])
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until_one_of(&[
            SyntaxKind::Colon,
            SyntaxKind::RightBrace,
            SyntaxKind::Comma,
        ]))
        .and_then(token(SyntaxKind::Colon))
        .map(|(((aliens0, name), aliens1), colon)| (aliens0, name, aliens1, colon))
        .and_then(eat_until_one_of(&[
            SyntaxKind::Identifier,
            SyntaxKind::RightBrace,
            SyntaxKind::Comma,
        ]))
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

fn parse_struct_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
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

fn parse_func_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::LeftBrace)
        .and_then(eat_whitespace())
        .and_then(zero_or_more(parse_expr()))
        .map(|((left_brace, aliens), exprs)| (left_brace, aliens, exprs))
        .and_then(eat_until(SyntaxKind::RightBrace))
        .map(|((left_brace, aliens1, exprs), aliens2)| (left_brace, aliens1, exprs, aliens2))
        .and_then(token(SyntaxKind::RightBrace))
        .map(|((left_brace, aliens1, exprs, aliens2), right_brace)| {
            (left_brace, aliens1, exprs, aliens2, right_brace)
        })
        .map(|(left_brace, aliens1, exprs, aliens2, right_brace)| {
            let mut elements = vec![];
            let span = left_brace.as_token().span();
            elements.push(left_brace);
            elements.extend(aliens1);
            elements.extend(exprs);
            elements.extend(aliens2);
            elements.push(right_brace);

            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BlockExpr, elements, span))
        })
}

fn parse_encoding<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::EncodingKw)
        .and_then(eat_until(SyntaxKind::ForKw))
        .and_then(token(SyntaxKind::ForKw))
        .map(|((enc, aliens1), for_)| (enc, aliens1, for_))
        .and_then(eat_until(SyntaxKind::Identifier))
        .map(|((enc, aliens1, for_), aliens2)| (enc, aliens1, for_, aliens2))
        .and_then(token(SyntaxKind::Identifier))
        .map(|((enc, aliens1, for_, aliens2), name)| (enc, aliens1, for_, aliens2, name))
        .and_then(eat_until(SyntaxKind::LeftBrace))
        .map(|((enc, aliens1, for_, aliens2, name), aliens3)| {
            (enc, aliens1, for_, aliens2, name, aliens3)
        })
        .and_then(parse_func_body())
        .map(|((enc, aliens1, for_, aliens2, name, aliens3), body)| {
            (enc, aliens1, for_, aliens2, name, aliens3, body)
        })
        .map(|(enc, aliens1, for_, aliens2, name, aliens3, body)| {
            let span = enc.as_token().span();
            let mut elements = vec![];
            elements.push(enc);
            elements.extend(aliens1);
            elements.push(for_);
            elements.extend(aliens2);
            let name_span = name.as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateName,
                vec![name],
                name_span,
            )));
            elements.extend(aliens3);
            elements.push(body);

            GreenNodeData::new(SyntaxKind::EncodingDecl, elements, span)
        })
}

fn parse_asm<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::AsmKw)
        .and_then(eat_until(SyntaxKind::ForKw))
        .and_then(token(SyntaxKind::ForKw))
        .map(|((asm_, aliens1), for_)| (asm_, aliens1, for_))
        .and_then(eat_until(SyntaxKind::Identifier))
        .map(|((asm_, aliens1, for_), aliens2)| (asm_, aliens1, for_, aliens2))
        .and_then(token(SyntaxKind::Identifier))
        .map(|((asm_, aliens1, for_, aliens2), name)| (asm_, aliens1, for_, aliens2, name))
        .and_then(eat_until(SyntaxKind::LeftBrace))
        .map(|((asm_, aliens1, for_, aliens2, name), aliens3)| {
            (asm_, aliens1, for_, aliens2, name, aliens3)
        })
        .and_then(parse_func_body())
        .map(|((asm_, aliens1, for_, aliens2, name, aliens3), body)| {
            (asm_, aliens1, for_, aliens2, name, aliens3, body)
        })
        .map(|(asm_, aliens1, for_, aliens2, name, aliens3, body)| {
            let span = asm_.as_token().span();
            let mut elements = vec![];
            elements.push(asm_);
            elements.extend(aliens1);
            elements.push(for_);
            elements.extend(aliens2);
            let name_span = name.as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateName,
                vec![name],
                name_span,
            )));
            elements.extend(aliens3);
            elements.push(body);

            GreenNodeData::new(SyntaxKind::AsmDecl, elements, span)
        })
}

fn parse_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    parse_binary_expr().or_else(parse_atom_expr())
}

fn parse_binary_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let operator_atom = token(SyntaxKind::At);
    let operator = eat_until_one_of(&[
        SyntaxKind::At,
        SyntaxKind::RightBrace,
        SyntaxKind::Semicolon,
    ])
    .and_then(operator_atom)
    .and_then(eat_until_one_of(&[
        SyntaxKind::At,
        SyntaxKind::Identifier,
        SyntaxKind::IntegerLiteral,
        SyntaxKind::BitLiteral,
        SyntaxKind::StringLiteral,
        SyntaxKind::Semicolon,
        SyntaxKind::RightBrace,
    ]));
    fold_left(
        parse_atom_expr(),
        operator,
        |left, ((aliens_left, op), aliens_right), right| {
            let span = op.as_token().span();
            let mut elements = vec![];
            elements.push(left);
            elements.extend(aliens_left);
            elements.push(op);
            elements.extend(aliens_right);
            elements.push(right);
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BinOpExpr, elements, span))
        },
    )
}

fn parse_field_access<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    fold_left(
        token(SyntaxKind::Identifier),
        token(SyntaxKind::Dot),
        |left, dot, right| {
            let span = dot.as_token().span();
            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::StructFieldAccess,
                vec![left, dot, right],
                span,
            ))
        },
    )
}

fn parse_atom_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let lit_atom = token(SyntaxKind::IntegerLiteral)
        .or_else(token(SyntaxKind::BitLiteral))
        .or_else(token(SyntaxKind::StringLiteral))
        .or_else(parse_field_access())
        .or_else(token(SyntaxKind::Identifier));
    eat_until_one_of(&[
        SyntaxKind::IntegerLiteral,
        SyntaxKind::BitLiteral,
        SyntaxKind::StringLiteral,
        SyntaxKind::Identifier,
        SyntaxKind::Semicolon,
        SyntaxKind::RightBrace,
    ])
    .and_then(lit_atom)
    .map(|(aliens, lit)| {
        let span = match lit {
            NodeOrToken::Node(ref n) => n.span(),
            NodeOrToken::Token(ref t) => t.span(),
        };
        let mut elements = aliens;
        elements.push(lit);
        NodeOrToken::Node(GreenNodeData::new(SyntaxKind::LiteralExpr, elements, span))
    })
}

fn parse_instr_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::InstrKw)
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::Identifier))
        .map(|((instr_kw, aliens1), name)| (instr_kw, aliens1, name))
        .and_then(eat_until(SyntaxKind::Colon))
        .map(|((instr_kw, aliens1, name), aliens2)| (instr_kw, aliens1, name, aliens2))
        .and_then(token(SyntaxKind::Colon))
        .map(|((instr_kw, aliens1, name, aliens2), colon)| {
            (instr_kw, aliens1, name, aliens2, colon)
        })
        .and_then(eat_until(SyntaxKind::Identifier))
        .map(|((instr_kw, aliens1, name, aliens2, colon), aliens3)| {
            (instr_kw, aliens1, name, aliens2, colon, aliens3)
        })
        .and_then(parse_template_instantiation())
        .map(
            |((instr_kw, aliens1, name, aliens2, colon, aliens3), template)| {
                (instr_kw, aliens1, name, aliens2, colon, aliens3, template)
            },
        )
        .and_then(eat_until(SyntaxKind::Semicolon))
        .map(
            |((instr_kw, aliens1, name, aliens2, colon, aliens3, template), aliens4)| {
                (
                    instr_kw, aliens1, name, aliens2, colon, aliens3, template, aliens4,
                )
            },
        )
        .and_then(token(SyntaxKind::Semicolon))
        .map(
            |((instr_kw, aliens1, name, aliens2, colon, aliens3, template, aliens4), semicolon)| {
                (
                    instr_kw, aliens1, name, aliens2, colon, aliens3, template, aliens4, semicolon,
                )
            },
        )
        .map(
            |(instr_kw, aliens1, name, aliens2, colon, aliens3, template, aliens4, semicolon)| {
                let mut elements = vec![];
                let span = instr_kw.as_token().span();
                elements.push(instr_kw);
                elements.extend(aliens1);
                let name_span = name.as_token().span();
                elements.push(NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::InstrName,
                    vec![name],
                    name_span,
                )));
                elements.extend(aliens2);
                elements.push(colon);
                elements.extend(aliens3);
                elements.push(template);
                elements.extend(aliens4);
                elements.push(semicolon);

                GreenNodeData::new(SyntaxKind::InstrDecl, elements, span)
            },
        )
}

fn parse_template_instantiation_param<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    eat_until_one_of(&[
        SyntaxKind::StringLiteral,
        SyntaxKind::IntegerLiteral,
        SyntaxKind::BitLiteral,
    ])
    .and_then(
        token(SyntaxKind::StringLiteral)
            .or_else(token(SyntaxKind::IntegerLiteral))
            .or_else(token(SyntaxKind::BitLiteral)),
    )
    .and_then(eat_until_one_of(&[
        SyntaxKind::Comma,
        SyntaxKind::RightAngle,
    ]))
    .map(|((aliens0, lit), aliens1)| {
        let mut elements = vec![];
        let span = lit.as_token().span();
        elements.extend(aliens0);
        let lit_span = lit.as_token().span();
        let lit_expr = NodeOrToken::Node(GreenNodeData::new(SyntaxKind::LiteralExpr, vec![lit], lit_span));
        elements.push(lit_expr);
        elements.extend(aliens1);
        NodeOrToken::Node(GreenNodeData::new(
            SyntaxKind::InstrParentTemplateArg,
            elements,
            span,
        ))
    })
}

fn parse_template_instantiation<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(eat_until(SyntaxKind::LeftAngle))
        .and_then(token(SyntaxKind::LeftAngle))
        .map(|((ident, aliens1), left_angle)| (ident, aliens1, left_angle))
        .and_then(separated(
            parse_template_instantiation_param(),
            token(SyntaxKind::Comma).map(|_| ()),
        ))
        .map(|((ident, aliens1, left_angle), params)| (ident, aliens1, left_angle, params))
        .and_then(eat_until(SyntaxKind::RightAngle))
        .map(|((ident, aliens1, left_angle, params), aliens2)| {
            (ident, aliens1, left_angle, params, aliens2)
        })
        .and_then(token(SyntaxKind::RightAngle))
        .map(
            |((ident, aliens1, left_angle, params, aliens2), right_angle)| {
                (ident, aliens1, left_angle, params, aliens2, right_angle)
            },
        )
        .map(
            |(ident, aliens1, left_angle, params, aliens2, right_angle)| {
                let mut elements = vec![];
                let span = ident.as_token().span();
                elements.push(NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::InstrParentTemplateName,
                    vec![ident],
                    span.clone(),
                )));
                elements.extend(aliens1);
                elements.push(left_angle);
                elements.extend(params);
                elements.extend(aliens2);
                elements.push(right_angle);

                NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::InstrParentTemplate,
                    elements,
                    span,
                ))
            },
        )
}
