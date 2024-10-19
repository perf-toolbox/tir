use lpl::syntax::GreenNodeData;
use lpl::{combinators::*, Span};
use lpl::{
    syntax::{GreenElement, NodeOrToken},
    ParseStream, Parser,
};

use crate::diagnostic::DiagKind;
use crate::{ImmElement, ImmNode, SyntaxKind, TokenStream};

pub fn parse(tokens: &[ImmElement]) -> ImmNode {
    let stream = TokenStream::new(tokens);

    let top_level_decl = instr_template_decl()
        .or_else(instr_decl())
        .or_else(encoding())
        .or_else(asm_())
        .or_else(enum_());
    let parser = zero_or_more(top_level_decl.map(NodeOrToken::Node).or_else(catch_all()));

    let result = parser.parse(stream).unwrap();

    GreenNodeData::new(SyntaxKind::TranslationUnit, result.0, Span::empty())
}

fn attached_comment<'a>() -> impl Parser<'a, TokenStream<'a>, Option<(ImmElement, ImmElement)>> {
    move |tokens: TokenStream<'a>| {
        if tokens.len() < 2 {
            return Err(DiagKind::UnexpectedEof(tokens.span()).into());
        }

        let maybe_comment = tokens.nth(0).unwrap();
        if maybe_comment.as_token().kind() != SyntaxKind::Comment {
            return Ok((None, Some(tokens)));
        }

        let maybe_newline = tokens.nth(1).unwrap();
        if maybe_newline.as_token().kind() != SyntaxKind::Whitespace
            && maybe_newline.as_token().text() != "\n"
        {
            return Err(DiagKind::TokenNotFound(SyntaxKind::Whitespace, tokens.span()).into());
        }

        Ok((Some((maybe_comment, maybe_newline)), tokens.slice(2..)))
    }
}

fn catch_all<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    move |tokens: TokenStream<'a>| {
        if tokens.len() > 0 {
            return Ok((tokens.nth(0).unwrap(), tokens.slice(1..)));
        }
        Err(DiagKind::UnexpectedEof(tokens.span()).into())
    }
}

fn eat_all<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<ImmElement>> {
    move |tokens: TokenStream<'a>| {
        let mut alien_tokens = vec![];
        for i in 0..tokens.len() {
            if let Some(element) = tokens.nth(i) {
                alien_tokens.push(element);
            }
        }

        Ok((alien_tokens, None))
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

        Err(DiagKind::TokenNotFound(kind, tokens.span()).into())
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

        Err(DiagKind::MultipleTokensNotFound(tokens.span()).into())
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

        Err(DiagKind::TokenNotFound(kind, tokens.span()).into())
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

        Err(DiagKind::MultipleTokensNotFound(tokens.span()).into())
    }
}

fn instr_template_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(isolate_until(
            token(SyntaxKind::LeftBrace).void(),
            token(SyntaxKind::InstrTemplateKw)
                .and_then(eat_until(SyntaxKind::Identifier))
                .and_then(token(SyntaxKind::Identifier))
                .and_then(eat_until(SyntaxKind::LeftAngle))
                .and_then(instr_template_parameters())
                .and_then(optional(
                    eat_until(SyntaxKind::Colon)
                        .and_then(token(SyntaxKind::Colon))
                        .and_then(eat_until(SyntaxKind::Identifier))
                        .and_then(template_instantiation()),
                ))
                .and_then(eat_all())
                .flat(),
        ))
        .map(
            |(comment, (kw, aliens1, name, aliens2, params, parent, aliens3))| {
                (comment, kw, aliens1, name, aliens2, params, parent, aliens3)
            },
        )
        .and_then(struct_body())
        .map(
            |((comment, kw, aliens1, name, aliens2, params, parent, aliens3), body)| {
                (
                    comment, kw, aliens1, name, aliens2, params, parent, aliens3, body,
                )
            },
        )
        .map(
            |(comment, kw, aliens1, name, aliens2, params, parent, aliens3, body)| {
                let mut elements = vec![];
                if let Some((comment, space)) = comment {
                    elements.push(comment);
                    elements.push(space);
                }
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
                if let Some((((aliens_parent1, colon), aliens_parent2), parent_inst)) = parent {
                    elements.extend(aliens_parent1);
                    elements.push(colon);
                    elements.extend(aliens_parent2);
                    elements.push(parent_inst);
                }
                elements.extend(aliens3);
                elements.push(NodeOrToken::Node(body));

                GreenNodeData::new(SyntaxKind::InstrTemplateDecl, elements, kw_span)
            },
        )
}

fn ty<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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
                    let lit_expr = NodeOrToken::Node(GreenNodeData::new(
                        SyntaxKind::LiteralExpr,
                        vec![lit],
                        lit_span,
                    ));
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

fn single_template_parameter<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    eat_until(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until(SyntaxKind::Colon))
        .and_then(token(SyntaxKind::Colon))
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(ty())
        .and_then(eat_until_one_of(&[
            SyntaxKind::Comma,
            SyntaxKind::RightAngle,
        ]))
        .flat()
        .map(|(aliens0, name, aliens1, colon, aliens2, ty, aliens3)| {
            let mut elements = vec![];
            let span = name.as_token().span();
            elements.extend(aliens0);
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateSingleParamName,
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

fn instr_template_parameters<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::LeftAngle)
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(separated(
            single_template_parameter(),
            token(SyntaxKind::Comma),
        ))
        .and_then(eat_until(SyntaxKind::RightAngle))
        .and_then(token(SyntaxKind::RightAngle))
        .flat()
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

fn struct_field<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    eat_until_one_of(&[SyntaxKind::Identifier, SyntaxKind::RightBrace])
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until_one_of(&[
            SyntaxKind::Colon,
            SyntaxKind::RightBrace,
            SyntaxKind::Comma,
        ]))
        .and_then(token(SyntaxKind::Colon))
        .and_then(eat_until_one_of(&[
            SyntaxKind::Identifier,
            SyntaxKind::RightBrace,
            SyntaxKind::Comma,
        ]))
        .and_then(ty())
        .and_then(eat_until_one_of(&[
            SyntaxKind::Comma,
            SyntaxKind::RightBrace,
        ]))
        .flat()
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

fn struct_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    isolate_block(
        token(SyntaxKind::LeftBrace).void(),
        token(SyntaxKind::RightBrace).void(),
        token(SyntaxKind::LeftBrace)
            .and_then(optional(separated(
                struct_field(),
                token(SyntaxKind::Comma),
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
            }),
    )
}

fn func_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    isolate_block(
        token(SyntaxKind::LeftBrace).void(),
        token(SyntaxKind::RightBrace).void(),
        token(SyntaxKind::LeftBrace)
            .and_then(eat_whitespace())
            .and_then(zero_or_more(expr()))
            .and_then(eat_until(SyntaxKind::RightBrace))
            .and_then(token(SyntaxKind::RightBrace))
            .flat()
            .map(|(left_brace, aliens1, exprs, aliens2, right_brace)| {
                let mut elements = vec![];
                let span = left_brace.as_token().span();
                elements.push(left_brace);
                elements.extend(aliens1);
                elements.extend(exprs);
                elements.extend(aliens2);
                elements.push(right_brace);

                NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BlockExpr, elements, span))
            }),
    )
}

fn encoding<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    isolate_until(
        token(SyntaxKind::LeftBrace).void(),
        token(SyntaxKind::EncodingKw)
            .and_then(eat_until(SyntaxKind::ForKw))
            .and_then(token(SyntaxKind::ForKw))
            .and_then(eat_until(SyntaxKind::Identifier))
            .and_then(token(SyntaxKind::Identifier))
            .flat()
            .and_then(eat_all()),
    )
    .map(|((enc, aliens1, for_, aliens2, name), aliens3)| {
        (enc, aliens1, for_, aliens2, name, aliens3)
    })
    .and_then(func_body())
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

fn asm_<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    isolate_until(
        token(SyntaxKind::LeftBrace).void(),
        token(SyntaxKind::AsmKw)
            .and_then(eat_until(SyntaxKind::ForKw))
            .and_then(token(SyntaxKind::ForKw))
            .and_then(eat_until(SyntaxKind::Identifier))
            .and_then(token(SyntaxKind::Identifier))
            .and_then(eat_all())
            .flat(),
    )
    .and_then(func_body())
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

fn expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    binary_expr().or_else(atom_expr())
}

fn binary_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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
        atom_expr(),
        operator,
        |left, ((aliens_left, op), aliens_right), right| {
            let span = op.as_token().span();
            let mut elements = vec![];
            let left_span = left.as_node().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::BinOpExprLeft,
                vec![left],
                left_span,
            )));
            elements.extend(aliens_left);
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::BinOpExprOp,
                vec![op],
                span.clone(),
            )));
            elements.extend(aliens_right);
            let right_span = right.as_node().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::BinOpExprRight,
                vec![right],
                right_span,
            )));
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BinOpExpr, elements, span))
        },
    )
}

fn field_access<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
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

fn atom_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let lit_atom = token(SyntaxKind::IntegerLiteral)
        .or_else(token(SyntaxKind::BitLiteral))
        .or_else(token(SyntaxKind::StringLiteral))
        .or_else(field_access())
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

fn instr_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(token(SyntaxKind::InstrKw))
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(eat_until(SyntaxKind::Colon))
        .and_then(token(SyntaxKind::Colon))
        .and_then(eat_until(SyntaxKind::Identifier))
        .and_then(template_instantiation())
        .and_then(eat_until(SyntaxKind::Semicolon))
        .and_then(token(SyntaxKind::Semicolon))
        .flat()
        .map(
            |(
                comment,
                instr_kw,
                aliens1,
                name,
                aliens2,
                colon,
                aliens3,
                template,
                aliens4,
                semicolon,
            )| {
                let mut elements = vec![];
                if let Some((comment, space)) = comment {
                    elements.push(comment);
                    elements.push(space);
                }
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

fn template_instantiation_param<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    eat_until_one_of(&[
        SyntaxKind::StringLiteral,
        SyntaxKind::IntegerLiteral,
        SyntaxKind::BitLiteral,
        SyntaxKind::Identifier,
    ])
    .and_then(
        token(SyntaxKind::StringLiteral)
            .or_else(token(SyntaxKind::IntegerLiteral))
            .or_else(token(SyntaxKind::BitLiteral))
            .or_else(token(SyntaxKind::Identifier)),
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
        let lit_expr = NodeOrToken::Node(GreenNodeData::new(
            SyntaxKind::LiteralExpr,
            vec![lit],
            lit_span,
        ));
        elements.push(lit_expr);
        elements.extend(aliens1);
        NodeOrToken::Node(GreenNodeData::new(
            SyntaxKind::InstrParentTemplateArg,
            elements,
            span,
        ))
    })
}

fn template_instantiation<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(eat_until(SyntaxKind::LeftAngle))
        .and_then(token(SyntaxKind::LeftAngle))
        .and_then(separated(
            template_instantiation_param(),
            token(SyntaxKind::Comma),
        ))
        .and_then(eat_until(SyntaxKind::RightAngle))
        .and_then(token(SyntaxKind::RightAngle))
        .flat()
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

fn enum_single_variant<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    isolate_until(
        token(SyntaxKind::RightBrace)
            .or_else(token(SyntaxKind::Comma))
            .void(),
        eat_until(SyntaxKind::Identifier)
            .and_then(token(SyntaxKind::Identifier))
            .and_then(eat_all())
            .map(|((aliens0, name), aliens1)| {
                let span = name.as_token().span();
                let mut elements = vec![];
                elements.extend(aliens0);
                elements.push(name);
                elements.extend(aliens1);

                NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::EnumVariantDecl,
                    elements,
                    span,
                ))
            }),
    )
}

fn enum_variants<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    isolate_block(
        token(SyntaxKind::LeftBrace).void(),
        token(SyntaxKind::RightBrace).void(),
        token(SyntaxKind::LeftBrace)
            .and_then(optional(separated(
                enum_single_variant(),
                token(SyntaxKind::Comma),
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

                NodeOrToken::Node(GreenNodeData::new(SyntaxKind::EnumBody, elements, span))
            }),
    )
}

fn enum_<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(isolate_until(
            token(SyntaxKind::LeftBrace).void(),
            token(SyntaxKind::EnumKw)
                .and_then(eat_until(SyntaxKind::Identifier))
                .and_then(token(SyntaxKind::Identifier))
                .and_then(eat_all())
                .flat(),
        ))
        .map(|(comment, (kw, aliens1, name, aliens2))| (comment, kw, aliens1, name, aliens2))
        .and_then(enum_variants())
        .map(|((comment, kw, aliens1, name, aliens2), variants)| {
            let span = kw.as_token().span();

            let mut elements = vec![];

            if let Some((comment, space)) = comment {
                elements.push(comment);
                elements.push(space);
            }

            elements.push(kw);
            elements.extend(aliens1);
            elements.push(name);
            elements.extend(aliens2);
            elements.push(variants);

            GreenNodeData::new(SyntaxKind::EnumDecl, elements, span)
        })
}
