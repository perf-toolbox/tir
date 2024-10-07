use std::marker::PhantomData;

use lpl::{
    syntax::{GreenNode, NodeOrToken},
    Span,
};

use crate::SyntaxKind;

pub type TranslationUnit = AstNode<TranslationUnitKind>;
pub type InstrTemplateDecl = AstNode<InstrTemplateDeclKind>;
pub type InstrTemplateParameterDecl = AstNode<InstrTemplateParameterDeclKind>;
pub type StructFieldDecl = AstNode<StructFieldDeclKind>;

pub struct AstNode<NodeKind: AstNodeKind> {
    green_node: GreenNode<SyntaxKind>,
    _p: PhantomData<NodeKind>,
}

impl TranslationUnit {
    pub fn instr_templates<'a>(&'a self) -> impl Iterator<Item = InstrTemplateDecl> + 'a {
        self.green_node
            .children()
            .iter()
            .filter_map(|child| match child {
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
}

impl InstrTemplateDecl {
    pub fn name<'a>(&'a self) -> &'a str {
        self.green_node
            .children()
            .iter()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::InstrTemplateName {
                        Some(node)
                    } else {
                        None
                    }
                }
                _ => None,
            });

        todo!()
    }

    pub fn template_parameters<'a>(&'a self) {
        todo!()
    }
}

impl<NK: AstNodeKind> AstNode<NK> {
    pub fn new(green_node: GreenNode<SyntaxKind>) -> Self {
        assert!(green_node.kind() == NK::get_syntax_kind());

        Self {
            green_node,
            _p: PhantomData::default(),
        }
    }

    pub fn span(&self) -> Span {
        self.green_node.span()
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
