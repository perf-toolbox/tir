use crate::lexer::lex;
use crate::token::{Token, TokenStream};
use crate::{
    BinaryOperationNode, BinaryOperator, CSTNode, ExpressionNode, InstrNode, InstrTemplateNode,
    PropertiesNode, PropertyNode, StructFieldNode, TemplateParameterNode, TypeNode,
};
use lpl::combinators::*;
use lpl::{ParseStream, Parser, ParserError, Spanned};

pub fn parse<'src>(input: &'src str) -> Result<Vec<CSTNode>, ParserError> {
    let tokens = lex(input)?;
    let stream = TokenStream::new(&tokens);

    // let parser = zero_or_more(parse_top_level()).and_then(eof());
    // let ((nodes, _), _) = parser.parse(stream)?;

    let parser = zero_or_more(parse_top_level());
    let (nodes, _) = parser.parse(stream)?;

    Ok(nodes)
}

fn token<'a, 'src>(expected: Token<'a>) -> impl Parser<'a, TokenStream<'a, 'src>, ()>
where
    'src: 'a,
    'a: 'src,
{
    move |input: TokenStream<'a, 'src>| {
        if let Some((token, span)) = input.peek() {
            if token == expected {
                Ok(((), input.slice(1..input.len())))
            } else {
                Err(ParserError::new("Unexpexpected token", span))
            }
        } else {
            Err(ParserError::new("Unexpected EOF", input.span()))
        }
    }
}

fn parse_top_level<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, CSTNode>
where
    'src: 'a,
    'a: 'src,
{
    parse_instr_template()
        .or_else(parse_properties())
        .or_else(parse_instr())
        .or_else(parse_comment())
}

fn parse_comment<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, CSTNode>
where
    'src: 'a,
    'a: 'src,
{
    move |input: TokenStream<'a, 'src>| {
        if let Some((token, span)) = input.peek() {
            if let Token::Comment(comment) = token {
                Ok((
                    CSTNode::Comment((comment.to_string(), span)),
                    input.slice(1..input.len()),
                ))
            } else {
                Err(ParserError::new("Unexpected token", span))
            }
        } else {
            Err(ParserError::new("Unexpected EOF", input.span()))
        }
    }
}

fn parse_instr_template<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, CSTNode>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::InstrTemplate)
        .and_then(parse_identifier())
        .and_then(parse_parameters())
        .and_then(parse_instr_body())
        .map(|(((_, name), params), body)| {
            CSTNode::InstrTemplate(Box::new(InstrTemplateNode {
                name: (name.0.to_string(), name.1),
                parameters: params,
                body,
            }))
        })
}

fn parse_properties<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, CSTNode>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::Properties)
        .and_then(token(Token::For))
        .and_then(parse_identifier())
        .and_then(parse_property_body())
        .map(|(((_, _), target), properties)| {
            CSTNode::Properties(Box::new(PropertiesNode {
                target: (target.0.to_string(), target.1),
                properties,
            }))
        })
}

fn parse_instr<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, CSTNode>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::Instr)
        .and_then(parse_identifier())
        .and_then(token(Token::Colon))
        .and_then(parse_identifier())
        .and_then(parse_arguments())
        .map(|((((_, name), _), template), args)| {
            CSTNode::Instr(Box::new(InstrNode {
                name: (name.0.to_string(), name.1),
                template: (template.0.to_string(), template.1),
                arguments: args,
            }))
        })
}

fn parse_identifier<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Spanned<&'src str>>
where
    'src: 'a,
    'a: 'src,
{
    move |input: TokenStream<'a, 'src>| {
        if let Some((token, span)) = input.peek() {
            if let Token::Identifier(ident) = token {
                Ok(((ident, input.span()), input.slice(1..input.len())))
            } else {
                Err(ParserError::new("Unexpected token", span))
            }
        } else {
            Err(ParserError::new("Unexpected EOF", input.span()))
        }
    }
}

fn parse_single_parameter<'a, 'src>(
) -> impl Parser<'a, TokenStream<'a, 'src>, TemplateParameterNode>
where
    'src: 'a,
    'a: 'src,
{
    parse_identifier()
        .and_then(token(Token::Colon))
        .and_then(parse_type())
        .map(|((name, _), type_)| TemplateParameterNode {
            name: (name.0.to_string(), name.1),
            type_,
        })
}

fn parse_parameters<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Vec<TemplateParameterNode>>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::LeftAngle)
        .and_then(separated(parse_single_parameter(), token(Token::Comma)))
        .and_then(token(Token::RightAngle))
        .map(|((_, params), _)| params)
}

fn parse_struct_item<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, StructFieldNode>
where
    'src: 'a,
    'a: 'src,
{
    parse_identifier()
        .and_then(token(Token::Colon))
        .and_then(parse_type())
        .map(|((name, _), type_)| StructFieldNode {
            name: (name.0.to_string(), name.1),
            type_,
        })
}

fn parse_instr_body<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Vec<CSTNode>>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::LeftBrace)
        .and_then(separated(
            parse_struct_item().map(|item| CSTNode::StructField(item)),
            token(Token::Comma),
        ))
        .and_then(token(Token::RightBrace))
        .map(|((_, items), _)| items)
}

fn parse_property_body<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Vec<PropertyNode>>
where
    'src: 'a,
    'a: 'src,
{
    todo()
}

fn parse_arguments<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Vec<ExpressionNode>>
where
    'src: 'a,
    'a: 'src,
{
    todo()
}

fn parse_number<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Spanned<i64>>
where
    'src: 'a,
    'a: 'src,
{
    move |input: TokenStream<'a, 'src>| {
        if let Some((token, span)) = input.peek() {
            if let Token::IntegerLiteral(num) = token {
                Ok(((num, input.span()), input.slice(1..input.len())))
            } else {
                Err(ParserError::new("Unexpected token", span))
            }
        } else {
            Err(ParserError::new("Unexpected EOF", input.span()))
        }
    }
}

fn parse_bitwidth<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, Spanned<i64>>
where
    'src: 'a,
    'a: 'src,
{
    token(Token::LeftAngle)
        .and_then(parse_number())
        .and_then(token(Token::RightAngle))
        .map(|((_, num), _)| num)
}

fn parse_type<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, TypeNode>
where
    'src: 'a,
    'a: 'src,
{
    parse_identifier()
        .and_then(optional(parse_bitwidth()))
        .try_map(|(name, bitwidth), _| {
            if name.0 == "bits" {
                if let Some(bitwidth) = bitwidth {
                    Ok(TypeNode::Bits(bitwidth))
                } else {
                    Err(ParserError::new("Missing bitwidth", name.1))
                }
            } else if name.0 == "Register" {
                Ok(TypeNode::Register)
            } else if name.0 == "str" {
                Ok(TypeNode::Str)
            } else {
                Err(ParserError::new("Unknown type", name.1))
            }
        })
}

fn parse_operand<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, ExpressionNode>
where
    'src: 'a,
    'a: 'src,
{
    separated(parse_identifier(), token(Token::Dot)).map(|name| {
        ExpressionNode::Path(
            name.iter()
                .map(|(name, span)| (name.to_string(), span.clone()))
                .collect(),
        )
    })
}

fn parse_bit_concat_expression<'a, 'src>() -> impl Parser<'a, TokenStream<'a, 'src>, ExpressionNode>
where
    'src: 'a,
    'a: 'src,
{
    parse_operand()
        .and_then(token(Token::At))
        .and_then(parse_operand())
        .map(|((left, _), right)| {
            ExpressionNode::BinaryOperation(Box::new(BinaryOperationNode {
                left: Box::new(left),
                operator: BinaryOperator::BitConcat,
                right: Box::new(right),
            }))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bits_type() {
        let input = "bits<32>";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        let result = parse_type().parse(stream).expect("Failed to parse type");
        if let TypeNode::Bits(bitwidth) = result.0 {
            assert_eq!(bitwidth.0, 32);
        } else {
            panic!("Expected bits type");
        }
    }

    #[test]
    fn test_parse_register_type() {
        let input = "Register";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        let result = parse_type().parse(stream).expect("Failed to parse type");
        if let TypeNode::Register = result.0 {
            assert!(true);
        } else {
            panic!("Expected register type");
        }
    }

    #[test]
    fn test_parse_template_parameter() {
        let input = "<$funct7: bits<7>, $funct3: bits<3>, $mnemonic: str>";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        let result = parse_parameters()
            .parse(stream)
            .expect("Failed to parse parameters");
        assert_eq!(result.0.len(), 3);
    }

    #[test]
    fn test_parse_instr_template_body() {
        let input = "{
            rd: Register,
            rs1: Register,
            rs2: Register,
        }";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        let result = parse_instr_body()
            .parse(stream)
            .expect("Failed to parse instr template body");
        assert_eq!(result.0.len(), 3);
    }

    #[test]
    fn test_parse_instr_template() {
        let input = "instr_template RInstr<$funct7: bits<7>, $funct3: bits<3>, $mnemonic: str> {
            rd: Register,
            rs1: Register,
            rs2: Register,
        }";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        parse_instr_template()
            .parse(stream)
            .expect("Failed to parse instr template");
    }

    #[test]
    fn test_parse_bit_concat_expression() {
        let input = "a @ b";
        let tokens = lex(input).unwrap();
        let stream = TokenStream::new(&tokens);
        parse_bit_concat_expression()
            .parse(stream)
            .expect("Failed to parse bit concat expression");
    }
}
