use core::fmt;
use std::marker::PhantomData;

use lpl::{syntax::NodeOrToken, Span};

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

pub type TranslationUnit = AstNode<TranslationUnitKind>;
pub type InstrTemplateDecl = AstNode<InstrTemplateDeclKind>;
pub type InstrTemplateParameterDecl = AstNode<InstrTemplateParameterDeclKind>;
pub type InstrDecl = AstNode<InstrDeclKind>;
pub type InstrTemplateArg = AstNode<InstrTemplateArgKind>;
pub type StructFieldDecl = AstNode<StructFieldDeclKind>;
pub type BlockExpr = AstNode<BlockExprKind>;
pub type LiteralExpr = AstNode<LiteralExprKind>;
pub type BinOpExpr = AstNode<BinOpExprKind>;
pub type EncodingDecl = AstNode<EncodingDeclKind>;
pub type AsmDecl = AstNode<AsmDeclKind>;

pub struct AstNode<NodeKind: AstNodeKind> {
    syntax_node: SyntaxNode,
    _p: PhantomData<NodeKind>,
}

pub fn build(root: SyntaxNode) -> TranslationUnit {
    TranslationUnit::new(root)
}

impl TranslationUnit {
    pub fn instr_templates<'a>(&'a self) -> impl Iterator<Item = InstrTemplateDecl> + 'a {
        self.syntax_node.children().filter_map(|child| match child {
            NodeOrToken::Node(node) => {
                if node.kind() == SyntaxKind::InstrTemplateDecl {
                    Some(InstrTemplateDecl::new(node.clone()))
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn instructions<'a>(&'a self) -> impl Iterator<Item = InstrDecl> + 'a {
        self.syntax_node.children().filter_map(|child| match child {
            NodeOrToken::Node(node) => {
                if node.kind() == SyntaxKind::InstrDecl {
                    Some(InstrDecl::new(node.clone()))
                } else {
                    None
                }
            }
            _ => None,
        })
    }
}

impl fmt::Debug for TranslationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TranslationUnit ")?;
        f.debug_list()
            .entries(self.instr_templates())
            .entries(self.instructions())
            .finish()
    }
}

impl InstrTemplateDecl {
    pub fn name<'a>(&'a self) -> String {
        self.syntax_node
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::InstrTemplateName {
                        Some(node)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .iter()
            .map(|node| node.children())
            .flatten()
            .find_map(|child| match child {
                crate::SyntaxElement::Token(token) => {
                    if token.kind() == SyntaxKind::Identifier {
                        Some(token.text().to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn template_parameters<'a>(&'a self) {
        todo!()
    }
}

impl fmt::Debug for InstrTemplateDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateDecl")
            .field("name", &self.name())
            .finish()
    }
}

impl InstrDecl {
    pub fn name<'a>(&'a self) -> String {
        self.syntax_node
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrName => Some(node),
                _ => None,
            })
            .iter()
            .flat_map(|node| node.children())
            .find_map(|child| match child {
                crate::SyntaxElement::Token(token) => {
                    if token.kind() == SyntaxKind::Identifier {
                        Some(token.text().to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn template_name<'a>(&'a self) -> String {
        self.syntax_node
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|node| node.children())
            .find_map(|child| match child {
                SyntaxElement::Node(node) if node.kind() == SyntaxKind::InstrParentTemplateName => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|node| node.children())
            .find_map(|child| match child {
                SyntaxElement::Token(token) if token.kind() == SyntaxKind::Identifier => {
                    Some(token.text().to_string())
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn template_args<'a>(&'a self) -> impl Iterator<Item = InstrTemplateArg> + 'a {
        self.syntax_node
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
                    Some(node)
                }
                _ => None,
            })
            // TODO(alexbatashev): can we get rid of `collect` here?
            .flat_map(|node| node.children().collect::<Vec<_>>())
            .filter_map(|child| match child {
                NodeOrToken::Node(child_node)
                    if child_node.kind() == SyntaxKind::InstrParentTemplateArg =>
                {
                    Some(InstrTemplateArg::new(child_node.clone()))
                }
                _ => None,
            })
    }
}

impl fmt::Debug for InstrDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrDecl")
            .field("name", &self.name())
            .field("template_name", &self.template_name())
            .field("template_args", &self.template_args().collect::<Vec<_>>())
            .finish()
    }
}

impl InstrTemplateArg {
    pub fn expr<'a>(&'a self) -> LiteralExpr {
        self.syntax_node
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::LiteralExpr => {
                    Some(LiteralExpr::new(node))
                }
                _ => None,
            })
            .unwrap()
    }
}

impl fmt::Debug for InstrTemplateArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateArg")
            .field("expr", &self.expr())
            .finish()
    }
}

impl LiteralExpr {
    pub fn value<'a>(&'a self) -> String {
        self.syntax_node
            .children()
            .find_map(|child| match child {
                NodeOrToken::Token(token)
                    if token.kind() == SyntaxKind::BitLiteral
                        || token.kind() == SyntaxKind::IntegerLiteral
                        || token.kind() == SyntaxKind::StringLiteral =>
                {
                    Some(token.text().to_string())
                }
                _ => None,
            })
            .unwrap()
    }
}

impl fmt::Debug for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiteralExpr")
            .field("value", &self.value())
            .finish()
    }
}

impl<NK: AstNodeKind> AstNode<NK> {
    pub fn new(syntax_node: SyntaxNode) -> Self {
        assert!(syntax_node.kind() == NK::get_syntax_kind());

        Self {
            syntax_node,
            _p: PhantomData::default(),
        }
    }

    pub fn span(&self) -> Span {
        self.syntax_node.span()
    }
}

pub trait AstNodeKind {
    fn get_syntax_kind() -> SyntaxKind;
}

pub enum TranslationUnitKind {}

impl AstNodeKind for TranslationUnitKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::TranslationUnit
    }
}

pub enum InstrTemplateDeclKind {}

impl AstNodeKind for InstrTemplateDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::InstrTemplateDecl
    }
}

pub enum InstrTemplateParameterDeclKind {}

impl AstNodeKind for InstrTemplateParameterDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::InstrTemplateSingleParam
    }
}

pub enum StructFieldDeclKind {}

impl AstNodeKind for StructFieldDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::StructField
    }
}

pub enum BlockExprKind {}

impl AstNodeKind for BlockExprKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::BlockExpr
    }
}

pub enum LiteralExprKind {}

impl AstNodeKind for LiteralExprKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::LiteralExpr
    }
}

pub enum BinOpExprKind {}

impl AstNodeKind for BinOpExprKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::BinOpExpr
    }
}

pub enum EncodingDeclKind {}

impl AstNodeKind for EncodingDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::EncodingDecl
    }
}

pub enum AsmDeclKind {}

impl AstNodeKind for AsmDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::AsmDecl
    }
}

pub enum InstrDeclKind {}

impl AstNodeKind for InstrDeclKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::InstrDecl
    }
}

pub enum InstrTemplateArgKind {}

impl AstNodeKind for InstrTemplateArgKind {
    fn get_syntax_kind() -> SyntaxKind {
        SyntaxKind::InstrParentTemplateArg
    }
}
