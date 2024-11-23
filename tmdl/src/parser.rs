use lang::{just_token, token, trivia};
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
        .or_else(enum_())
        .or_else(impl_())
        .or_else(flag());
    let parser = zero_or_more(top_level_decl.map(NodeOrToken::Node).or_else(catch_all()));

    let result = parser.parse(stream).unwrap();

    GreenNodeData::new(SyntaxKind::TranslationUnit, result.0, Span::empty())
}

fn attached_comment<'a>() -> impl Parser<'a, TokenStream<'a>, Vec<ImmElement>> {
    move |tokens: TokenStream<'a>| {
        if tokens.len() < 2 {
            return Err(DiagKind::UnexpectedEof(tokens.span()).into());
        }

        let mut comments = vec![];

        for i in (0..tokens.len()).step_by(2) {
            let maybe_comment = tokens.nth(i).unwrap();
            if maybe_comment.as_token().kind() != SyntaxKind::Comment
                && maybe_comment.as_token().kind() != SyntaxKind::LocalDocComment
            {
                break;
            }

            let maybe_newline = tokens.nth(i + 1).unwrap();
            if maybe_newline.as_token().kind() != SyntaxKind::Whitespace
                && maybe_newline.as_token().text() != "\n"
            {
                return Err(DiagKind::TokenNotFound(SyntaxKind::Whitespace, tokens.span()).into());
            }

            comments.push(maybe_comment);
            comments.push(maybe_newline);
        }

        let len = comments.len();
        Ok((comments, tokens.slice(len..)))
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

fn instr_template_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    let parent_template = token(SyntaxKind::Colon).and_then(template_instantiation());
    attached_comment()
        .and_then(just_token(SyntaxKind::InstrTemplateKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(instr_template_parameters())
        .and_then(optional(parent_template))
        .and_then(trivia())
        .and_then(struct_body())
        .flat()
        .map(|(comment, kw, name, params, parent, aliens, body)| {
            let mut elements = comment;

            let kw_span = kw.as_token().span();
            elements.push(kw);
            elements.extend(name.trivia().iter().cloned());
            let name_span = name.token().as_token().span();
            elements.push(GreenElement::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateName,
                vec![name.token().clone()],
                name_span,
            )));
            elements.push(NodeOrToken::Node(params));
            if let Some((colon, parent_inst)) = parent {
                elements.extend(colon.trivia().iter().cloned());
                elements.push(colon.token().clone());
                elements.push(parent_inst);
            }
            elements.extend(aliens);
            elements.push(NodeOrToken::Node(body));

            GreenNodeData::new(SyntaxKind::InstrTemplateDecl, elements, kw_span)
        })
        .label("instr_template_decl")
}

fn ty<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let params = move || {
        just_token(SyntaxKind::LeftAngle)
            .and_then(just_token([
                SyntaxKind::IntegerLiteral,
                SyntaxKind::StringLiteral,
                SyntaxKind::BitLiteral,
            ]))
            .and_then(just_token(SyntaxKind::RightAngle))
            .flat()
            .map(|(left_angle, lit, right_angle)| {
                let span = left_angle.as_token().span();
                let lit_span = lit.as_token().span();
                let lit_expr = NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::LiteralExpr,
                    vec![lit],
                    lit_span,
                ));
                let elements = vec![left_angle, lit_expr, right_angle];
                NodeOrToken::Node(GreenNodeData::new(SyntaxKind::TypeParams, elements, span))
            })
            .label("type params")
    };
    let single_type = token(SyntaxKind::Identifier)
        .and_then(optional(params()))
        .map(|(ident, params)| {
            let span = ident.token().as_token().span();
            let mut elements = vec![];
            elements.extend(ident.trivia().iter().cloned());
            elements.push(ident.token().clone());
            if let Some(params) = params {
                elements.push(params)
            }
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::Type, elements, span))
        })
        .label("plain type");

    let array = token(SyntaxKind::LeftBracket)
        .and_then(token(SyntaxKind::Identifier))
        .and_then(optional(params()))
        .and_then(token(SyntaxKind::RightBracket))
        .flat()
        .map(|(left_bracket, ident, params, right_bracket)| {
            let span = ident.token().as_token().span();
            let mut elements = vec![];
            elements.extend(left_bracket.trivia().iter().cloned());
            elements.push(left_bracket.token().clone());
            elements.extend(ident.trivia().iter().cloned());
            elements.push(ident.token().clone());
            if let Some(params) = params {
                elements.push(params)
            }
            elements.extend(right_bracket.trivia().iter().cloned());
            elements.push(right_bracket.token().clone());
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::Type, elements, span))
        })
        .label("array type");

    single_type.or_else(array).label("type")
}

fn single_template_parameter<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Colon))
        .and_then(ty())
        .and_then(trivia())
        .flat()
        .map(|(name, colon, ty, aliens)| {
            let mut elements = vec![];
            let span = name.token().as_token().span();
            elements.extend(name.trivia().iter().cloned());
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateSingleParamName,
                vec![name.token().clone()],
                span.clone(),
            )));
            elements.extend(colon.trivia().iter().cloned());
            elements.push(colon.token().clone());
            elements.push(ty);
            elements.extend(aliens);
            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrTemplateSingleParam,
                elements,
                span,
            ))
        })
        .label("single template parameter")
}

fn instr_template_parameters<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::LeftAngle)
        .and_then(separated(
            single_template_parameter(),
            just_token(SyntaxKind::Comma),
        ))
        .and_then(token(SyntaxKind::RightAngle))
        .flat()
        .map(|(angle_start, params, angle_end)| {
            let mut elements = vec![];
            let span = angle_start.token().as_token().span();
            elements.extend(angle_start.trivia().iter().cloned());
            elements.push(angle_start.token().clone());
            elements.extend(params);
            elements.extend(angle_end.trivia().iter().cloned());
            elements.push(angle_end.token().clone());
            GreenNodeData::new(SyntaxKind::InstrTemplateParams, elements, span)
        })
        .label("instr template parameters")
}

fn struct_field<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Colon))
        .and_then(ty())
        .and_then(trivia())
        .flat()
        .map(|(name, colon, ty, aliens)| {
            let span = name.token().as_token().span();
            let mut elements = vec![];
            elements.extend(name.trivia().iter().cloned());
            elements.push(name.token().clone());
            elements.extend(colon.trivia().iter().cloned());
            elements.push(colon.token().clone());
            elements.push(ty);
            elements.extend(aliens);

            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::StructField, elements, span))
        })
        .label("struct field")
}

fn struct_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    just_token(SyntaxKind::LeftBrace)
        .and_then(optional(separated(
            struct_field(),
            just_token(SyntaxKind::Comma),
        )))
        .and_then(token(SyntaxKind::RightBrace))
        .flat()
        .map(|(left_brace, fields, right_brace)| {
            let span = left_brace.as_token().span();
            let mut elements = vec![];
            elements.push(left_brace);
            if let Some(fields) = fields {
                elements.extend(fields);
            }
            elements.extend(right_brace.trivia().iter().cloned());
            elements.push(right_brace.token().clone());

            GreenNodeData::new(SyntaxKind::StructBody, elements, span)
        })
        .label("struct body")
}

fn func_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    just_token(SyntaxKind::LeftBrace)
        .and_then(zero_or_more(expr()))
        .and_then(token(SyntaxKind::RightBrace))
        .flat()
        .map(|(left_brace, exprs, right_brace)| {
            let mut elements = vec![];
            let span = left_brace.as_token().span();
            elements.push(left_brace);
            elements.extend(exprs);
            elements.extend(right_brace.trivia().iter().cloned());
            elements.push(right_brace.token().clone());

            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BlockExpr, elements, span))
        })
        .label("func body")
}

fn encoding<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    just_token(SyntaxKind::EncodingKw)
        .and_then(token(SyntaxKind::ForKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(trivia())
        .and_then(func_body())
        .flat()
        .map(|(kw, for_kw, name, aliens, body)| {
            let span = kw.as_token().span();
            let mut elements = vec![];
            elements.push(kw);
            elements.extend(for_kw.trivia().iter().cloned());
            elements.push(for_kw.token().clone());
            elements.extend(name.trivia().iter().cloned());
            let name_span = name.token().as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::ImplTargetName,
                vec![name.token().clone()],
                name_span,
            )));
            elements.extend(aliens);
            elements.push(body);

            GreenNodeData::new(SyntaxKind::EncodingDecl, elements, span)
        })
        .label("encoding")
}

fn asm_<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    just_token(SyntaxKind::AsmKw)
        .and_then(token(SyntaxKind::ForKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(trivia())
        .and_then(func_body())
        .flat()
        .map(|(asm_, for_, name, aliens, body)| {
            let span = asm_.as_token().span();
            let mut elements = vec![];
            elements.push(asm_);
            elements.extend(for_.trivia().iter().cloned());
            elements.push(for_.token().clone());
            elements.extend(name.trivia().iter().cloned());
            let name_span = name.token().as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::ImplTargetName,
                vec![name.token().clone()],
                name_span,
            )));
            elements.extend(aliens);
            elements.push(body);

            GreenNodeData::new(SyntaxKind::AsmDecl, elements, span)
        })
        .label("asm")
}

fn expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    recursive(|expr| {
        let inline_expr = recursive(|inline_expr| {
            let literal_expr = move || {
                token(SyntaxKind::BitLiteral)
                    .or_else(token(SyntaxKind::StringLiteral))
                    .or_else(token(SyntaxKind::IntegerLiteral))
                    .or_else(token(SyntaxKind::SelfKw))
                    .or_else(token(SyntaxKind::Identifier))
                    .map(|lit| {
                        let mut elements = vec![];

                        elements.extend(lit.trivia().iter().cloned());
                        elements.push(lit.token().clone());

                        NodeOrToken::Node(GreenNodeData::new(
                            SyntaxKind::LiteralExpr,
                            elements,
                            lit.token().as_token().span(),
                        ))
                    })
            };

            let list = token(SyntaxKind::LeftBracket)
                .and_then(separated(inline_expr, just_token(SyntaxKind::Comma)))
                .and_then(token(SyntaxKind::RightBracket))
                .flat()
                .map(|(left_bracket, items, right_bracket)| {
                    let mut elements = vec![];

                    elements.extend(left_bracket.trivia().iter().cloned());
                    elements.push(left_bracket.token().clone());

                    elements.extend(items);

                    elements.extend(right_bracket.trivia().iter().cloned());
                    elements.push(right_bracket.token().clone());

                    NodeOrToken::Node(GreenNodeData::new(
                        SyntaxKind::ListExpr,
                        elements,
                        left_bracket.token().as_token().span(),
                    ))
                });

            let field_expr = move || {
                recursive(|field_expr| {
                    literal_expr()
                        .and_then(just_token(SyntaxKind::Dot))
                        .and_then(field_expr)
                        .flat()
                        .map(|(left, dot, right): (ImmElement, ImmElement, ImmElement)| {
                            let span = dot.as_token().span();
                            let node = NodeOrToken::Node(GreenNodeData::new(
                                SyntaxKind::FieldExpr,
                                vec![left, dot, right],
                                span.clone(),
                            ));
                            NodeOrToken::Node(GreenNodeData::new(
                                SyntaxKind::LiteralExpr,
                                vec![node],
                                span,
                            ))
                        })
                })
            };
            // let field_expr = move || {
            //     fold_left(
            //         just_token([SyntaxKind::Identifier, SyntaxKind::SelfKw]),
            //         just_token(SyntaxKind::Dot),
            //         |left, dot, right| {
            //             let span = dot.as_token().span();
            //             let node = NodeOrToken::Node(GreenNodeData::new(
            //                 SyntaxKind::FieldExpr,
            //                 vec![left, dot, right],
            //                 span.clone(),
            //             ));
            //             NodeOrToken::Node(GreenNodeData::new(
            //                 SyntaxKind::LiteralExpr,
            //                 vec![node],
            //                 span,
            //             ))
            //         },
            //     )
            //     .label("field expr")
            // };

            let atom = move || literal_expr().or_else(field_expr());

            let bit_concat = fold_left(atom(), token(SyntaxKind::At), |left, op, right| {
                let span = op.token().as_token().span();
                let mut elements = vec![];
                let left_span = left.as_node().span();
                elements.push(NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::BinOpExprLeft,
                    vec![left],
                    left_span,
                )));
                elements.extend(op.trivia().iter().cloned());
                elements.push(NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::BinOpExprOp,
                    vec![op.token().clone()],
                    span.clone(),
                )));
                let right_span = right.as_node().span();
                elements.push(NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::BinOpExprRight,
                    vec![right],
                    right_span,
                )));
                NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BinOpExpr, elements, span))
            });

            bit_concat.or_else(atom()).or_else(list)
        });

        inline_expr
    })
}

// fn binary_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
//     let operator_atom = token(SyntaxKind::At);
//     let operator = operator_atom.and_then(trivia());
//     fold_left(atom_expr(), operator, |left, (op, aliens_right), right| {
//         let span = op.token().as_token().span();
//         let mut elements = vec![];
//         let left_span = left.as_node().span();
//         elements.push(NodeOrToken::Node(GreenNodeData::new(
//             SyntaxKind::BinOpExprLeft,
//             vec![left],
//             left_span,
//         )));
//         elements.extend(op.trivia().iter().cloned());
//         elements.push(NodeOrToken::Node(GreenNodeData::new(
//             SyntaxKind::BinOpExprOp,
//             vec![op.token().clone()],
//             span.clone(),
//         )));
//         elements.extend(aliens_right);
//         let right_span = right.as_node().span();
//         elements.push(NodeOrToken::Node(GreenNodeData::new(
//             SyntaxKind::BinOpExprRight,
//             vec![right],
//             right_span,
//         )));
//         NodeOrToken::Node(GreenNodeData::new(SyntaxKind::BinOpExpr, elements, span))
//     })
//     .label("binary expr")
// }

// fn field_access<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
//     fold_left(
//         just_token([SyntaxKind::Identifier, SyntaxKind::SelfKw]),
//         just_token(SyntaxKind::Dot),
//         |left, dot, right| {
//             let span = dot.as_token().span();
//             let node = NodeOrToken::Node(GreenNodeData::new(
//                 SyntaxKind::StructFieldAccess,
//                 vec![left, dot, right],
//                 span.clone(),
//             ));
//             NodeOrToken::Node(GreenNodeData::new(
//                 SyntaxKind::LiteralExpr,
//                 vec![node],
//                 span,
//             ))
//         },
//     )
//     .map(|element| match &element {
//         NodeOrToken::Node(_) => element,
//         NodeOrToken::Token(t) => NodeOrToken::Node(GreenNodeData::new(
//             SyntaxKind::LiteralExpr,
//             vec![element.clone()],
//             t.span(),
//         )),
//     })
//     .label("field access")
// }

fn atom_expr<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let map = |lit: lang::WrappedToken<SyntaxKind>| {
        let mut elements = vec![];

        elements.extend(lit.trivia().iter().cloned());
        elements.push(lit.token().clone());

        NodeOrToken::Node(GreenNodeData::new(
            SyntaxKind::LiteralExpr,
            elements,
            lit.token().as_token().span(),
        ))
    };

    // FIXME fold_left causes tokens to be skipped sometimes
    let lit_atom = token([
        SyntaxKind::IntegerLiteral,
        SyntaxKind::BitLiteral,
        SyntaxKind::StringLiteral,
        SyntaxKind::Identifier,
        SyntaxKind::SelfKw,
    ])
    .map(map)
    .label("literal atom");
    // field_access().or_else(lit_atom).label("atom expr")
    lit_atom
}

fn instr_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(just_token(SyntaxKind::InstrKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::Colon))
        .and_then(template_instantiation())
        .and_then(token(SyntaxKind::Semicolon))
        .flat()
        .map(|(comment, instr_kw, name, colon, template, semicolon)| {
            let mut elements = comment;

            let span = instr_kw.as_token().span();
            elements.push(instr_kw);
            let name_span = name.token().as_token().span();
            elements.extend(name.trivia().iter().cloned());
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrName,
                vec![name.token().clone()],
                name_span,
            )));
            elements.extend(colon.trivia().iter().cloned());
            elements.push(colon.token().clone());
            elements.push(template);
            elements.extend(semicolon.trivia().iter().cloned());
            elements.push(semicolon.token().clone());

            GreenNodeData::new(SyntaxKind::InstrDecl, elements, span)
        })
        .label("instr decl")
}

fn template_instantiation_param<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    // TODO parse trivia and attach to the element
    token(SyntaxKind::StringLiteral)
        .or_else(token(SyntaxKind::IntegerLiteral))
        .or_else(token(SyntaxKind::BitLiteral))
        .or_else(token(SyntaxKind::Identifier))
        .map(|lit| {
            let mut elements = vec![];
            let span = lit.token().as_token().span();
            elements.extend(lit.trivia().iter().cloned());
            let lit_expr = NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::LiteralExpr,
                vec![lit.token().clone()],
                span.clone(),
            ));
            elements.push(lit_expr);
            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrParentTemplateArg,
                elements,
                span,
            ))
        })
        .label("template inst param")
}

fn template_instantiation<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::LeftAngle))
        .and_then(separated(
            template_instantiation_param(),
            just_token(SyntaxKind::Comma),
        ))
        .and_then(token(SyntaxKind::RightAngle))
        .flat()
        .map(|(ident, left_angle, params, right_angle)| {
            let mut elements = vec![];
            let span = ident.token().as_token().span();
            elements.extend(ident.trivia().iter().cloned());
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrParentTemplateName,
                vec![ident.token().clone()],
                span.clone(),
            )));
            elements.extend(left_angle.trivia().iter().cloned());
            elements.push(left_angle.token().clone());
            elements.extend(params);
            elements.extend(right_angle.trivia().iter().cloned());
            elements.push(right_angle.token().clone());

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::InstrParentTemplate,
                elements,
                span,
            ))
        })
        .label("template inst")
}

fn enum_single_variant<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Identifier)
        .and_then(trivia())
        .map(|(name, trivia)| {
            let span = name.token().as_token().span();
            let mut elements = vec![];
            elements.extend(name.trivia().iter().cloned());
            elements.push(name.token().clone());
            elements.extend(trivia);

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::EnumVariantDecl,
                elements,
                span,
            ))
        })
        .label("enum single variant")
}

fn enum_variants<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::LeftBrace)
        .and_then(optional(separated(
            enum_single_variant(),
            just_token(SyntaxKind::Comma),
        )))
        .and_then(token(SyntaxKind::RightBrace))
        .flat()
        .map(|(left_brace, fields, right_brace)| {
            let span = left_brace.token().as_token().span();
            let mut elements = vec![];
            elements.extend(left_brace.trivia().iter().cloned());
            elements.push(left_brace.token().clone());
            if let Some(fields) = fields {
                elements.extend(fields);
            }
            elements.extend(right_brace.trivia().iter().cloned());
            elements.push(right_brace.token().clone());

            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::EnumBody, elements, span))
        })
        .label("enum variants")
}

fn enum_<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(token(SyntaxKind::EnumKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(enum_variants())
        .flat()
        .map(|(comment, kw, name, body)| {
            let span = kw.token().as_token().span();

            let mut elements = comment;

            elements.extend(kw.trivia().iter().cloned());
            elements.push(kw.token().clone());
            elements.extend(name.trivia().iter().cloned());
            elements.push(name.token().clone());
            elements.push(body);

            GreenNodeData::new(SyntaxKind::EnumDecl, elements, span)
        })
        .label("enum")
}

fn impl_body<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::LeftBrace)
        .and_then(zero_or_more(func_decl()))
        .and_then(token(SyntaxKind::RightBrace))
        .flat()
        .map(|(left, functions, right)| {
            let mut elements = vec![];
            let span = left.token().as_token().span();
            elements.extend(left.trivia().iter().cloned());
            elements.push(left.token().clone());
            elements.extend(functions);
            elements.extend(right.trivia().iter().cloned());
            elements.push(right.token().clone());
            NodeOrToken::Node(GreenNodeData::new(SyntaxKind::ImplBody, elements, span))
        })
        .label("trait impl body")
}

fn impl_<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    token(SyntaxKind::ImplKw)
        .and_then(token(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::ForKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(impl_body())
        .flat()
        .map(|(kw, trait_name, for_kw, target_name, body)| {
            let span = kw.token().as_token().span();

            let mut elements = vec![];

            elements.extend(kw.trivia().iter().cloned());
            elements.push(kw.token().clone());
            elements.extend(trait_name.trivia().iter().cloned());
            let name_span = trait_name.token().as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::ImplTraitName,
                vec![trait_name.token().clone()],
                name_span,
            )));
            elements.extend(for_kw.trivia().iter().cloned());
            elements.push(for_kw.token().clone());
            elements.extend(target_name.trivia().iter().cloned());
            let target_span = target_name.token().as_token().span();
            elements.push(NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::ImplTargetName,
                vec![target_name.token().clone()],
                target_span,
            )));
            elements.push(body);

            GreenNodeData::new(SyntaxKind::ImplDecl, elements, span)
        })
        .label("trait impl")
}

fn flag<'a>() -> impl Parser<'a, TokenStream<'a>, ImmNode> {
    attached_comment()
        .and_then(just_token(SyntaxKind::FlagKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::Semicolon))
        .flat()
        .map(|(comments, flag_kw, name, semicolon)| {
            let mut elements = comments;

            let span = flag_kw.as_token().span();
            elements.push(flag_kw);
            elements.extend(name.trivia().iter().cloned());
            elements.push(name.token().clone());
            elements.extend(semicolon.trivia().iter().cloned());
            elements.push(semicolon.token().clone());

            GreenNodeData::new(SyntaxKind::FlagDecl, elements, span)
        })
        .label("flag")
}

fn func_param<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    let generic_param = token(SyntaxKind::Identifier)
        .and_then(token(SyntaxKind::Colon))
        .and_then(ty())
        .and_then(trivia())
        .flat()
        .map(|(name, colon, ty, aliens)| {
            let mut elements = vec![];

            elements.extend(name.trivia().iter().cloned());
            elements.push(name.token().clone());
            elements.extend(colon.trivia().iter().cloned());
            elements.push(colon.token().clone());
            elements.push(ty);
            elements.extend(aliens);

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::FnParam,
                elements,
                name.token().as_token().span(),
            ))
        });

    let self_param = token(SyntaxKind::SelfKw)
        .and_then(trivia())
        .map(|(kw, aliens)| {
            let mut elements = vec![];

            elements.extend(kw.trivia().iter().cloned());
            elements.push(kw.token().clone());
            elements.extend(aliens);

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::FnParam,
                elements,
                kw.token().as_token().span(),
            ))
        });

    self_param
        .or_else(generic_param)
        .label("function parameter")
}

fn func_ret_ty<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    token(SyntaxKind::Minus)
        .and_then(just_token(SyntaxKind::RightAngle))
        .and_then(ty())
        .flat()
        .map(|(minus, _angle, ty)| {
            let mut elements = vec![];

            elements.extend(minus.trivia().iter().cloned());

            let arrow = NodeOrToken::Token(
                lpl::syntax::GreenTokenData::new(SyntaxKind::Arrow, "->".to_string())
                    .spanned(minus.token().as_token().span()),
            );
            elements.push(arrow);

            elements.push(ty);

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::FnRetType,
                elements,
                minus.token().as_token().span(),
            ))
        })
        .label("function return type")
}

fn func_signature<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    attached_comment()
        .and_then(token(SyntaxKind::FnKw))
        .and_then(token(SyntaxKind::Identifier))
        .and_then(token(SyntaxKind::LeftParen))
        .and_then(optional(separated(
            func_param(),
            just_token(SyntaxKind::Comma),
        )))
        .and_then(token(SyntaxKind::RightParen))
        .and_then(trivia())
        .and_then(optional(func_ret_ty()))
        .flat()
        .map(
            |(comment, kw, name, left_paren, params, right_paren, aliens, ret_ty)| {
                let mut elements = comment;

                elements.extend(kw.trivia().iter().cloned());
                elements.push(kw.token().clone());
                elements.extend(name.trivia().iter().cloned());
                elements.push(name.token().clone());
                elements.extend(left_paren.trivia().iter().cloned());
                elements.push(left_paren.token().clone());

                if let Some(params) = params {
                    elements.extend(params);
                }

                elements.extend(right_paren.trivia().iter().cloned());
                elements.push(right_paren.token().clone());

                elements.extend(aliens);

                if let Some(ret_ty) = ret_ty {
                    elements.push(ret_ty);
                }

                NodeOrToken::Node(GreenNodeData::new(
                    SyntaxKind::FnSignature,
                    elements,
                    kw.token().as_token().span(),
                ))
            },
        )
        .label("function signature")
}

fn func_decl<'a>() -> impl Parser<'a, TokenStream<'a>, ImmElement> {
    func_signature()
        .and_then(trivia())
        .and_then(func_body())
        .flat()
        .map(|(sig, aliens, body)| {
            let mut elements = vec![];

            elements.push(sig);
            elements.extend(aliens);
            elements.push(body);

            NodeOrToken::Node(GreenNodeData::new(
                SyntaxKind::FnDecl,
                elements,
                // FIXME add a correct span
                Span::empty(),
            ))
        })
        .label("function declaration")
}
