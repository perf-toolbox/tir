use core::fmt;

use lpl::{syntax::NodeOrToken, Span};

use crate::{SyntaxKind, SyntaxNode};

pub trait ASTNode {
    fn syntax(&self) -> &SyntaxNode;
    fn span(&self) -> Span;
}

#[derive(Clone)]
pub enum Type {
    Register,
    Bits(u16),
    String,
}

#[derive(Clone)]
pub enum ASTNodeKind {
    SourceFile(SourceFile),
    InstrTemplateDecl(InstrTemplateDecl),
    InstrTemplateParameterDecl(InstrTemplateParameterDecl),
    InstrDecl(InstrDecl),
    InstrTemplateArg(InstrTemplateArg),
    StructFieldDecl(StructFieldDecl),
    Expr(Expr),
    EncodingDecl(EncodingDecl),
    AsmDecl(AsmDecl),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal,
}

#[derive(Clone)]
pub struct SourceFile {
    syntax: SyntaxNode,
    instr_templates: Vec<ASTNodeKind>,
    instructions: Vec<ASTNodeKind>,
    encodings: Vec<ASTNodeKind>,
    asm_decls: Vec<ASTNodeKind>,
}

#[derive(Clone)]
pub struct InstrTemplateDecl {
    syntax: SyntaxNode,
    params: Vec<ASTNodeKind>,
    fields: Vec<ASTNodeKind>,
}

#[derive(Clone)]
pub struct InstrTemplateParameterDecl {
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub struct InstrDecl {
    syntax: SyntaxNode,
    template_args: Vec<ASTNodeKind>,
}

#[derive(Clone)]
pub struct InstrTemplateArg {
    #[allow(dead_code)]
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub struct EncodingDecl {
    #[allow(dead_code)]
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub struct AsmDecl {
    #[allow(dead_code)]
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub struct StructFieldDecl {
    syntax: SyntaxNode,
}

impl Type {
    pub fn new(syntax: SyntaxNode) -> Option<Type> {
        if syntax.kind() != SyntaxKind::Type {
            return None;
        }

        let ident = syntax
            .children()
            .find(|c| {
                if let NodeOrToken::Token(token) = c {
                    token.kind() == SyntaxKind::Identifier
                } else {
                    false
                }
            })?
            .as_token()
            .text()
            .to_string();

        match ident.as_ref() {
            "str" => Some(Type::String),
            "Register" => Some(Type::Register),
            "bits" => {
                let param = syntax
                    .children()
                    .find_map(|c| match c {
                        NodeOrToken::Node(node) if node.kind() == SyntaxKind::TypeParams => {
                            Some(node)
                        }
                        _ => None,
                    })
                    .iter()
                    .flat_map(|n| n.children())
                    .find_map(|c| match c {
                        NodeOrToken::Node(node) if node.kind() == SyntaxKind::LiteralExpr => {
                            Some(node)
                        }
                        _ => None,
                    })
                    .iter()
                    .flat_map(|n| n.children())
                    .find_map(|c| match c {
                        NodeOrToken::Token(token) if token.kind() == SyntaxKind::IntegerLiteral => {
                            Some(token.text().to_string())
                        }
                        _ => None,
                    })?;

                let num_bits = u16::from_str_radix(&param, 10).ok()?;

                Some(Type::Bits(num_bits))
            }
            _ => unreachable!("Unknown type `{}`", ident),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Register => write!(f, "Register"),
            Type::String => write!(f, "str"),
            Type::Bits(num) => write!(f, "bits<{}>", num),
        }
    }
}

impl SourceFile {
    pub fn new(root: SyntaxNode) -> Option<ASTNodeKind> {
        if root.kind() != SyntaxKind::TranslationUnit {
            return None;
        }

        let instr_templates = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrTemplateDecl => {
                    InstrTemplateDecl::new(node.clone())
                }
                _ => None,
            })
            .collect();

        let instructions = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrDecl => {
                    InstrDecl::new(node.clone())
                }
                _ => None,
            })
            .collect();

        let encodings = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::EncodingDecl => {
                    EncodingDecl::new(node.clone())
                }
                _ => None,
            })
            .collect();

        let asm_decls = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::AsmDecl => {
                    AsmDecl::new(node.clone())
                }
                _ => None,
            })
            .collect();

        Some(ASTNodeKind::SourceFile(SourceFile {
            syntax: root,
            instr_templates,
            instructions,
            encodings,
            asm_decls,
        }))
    }
}

impl ASTNode for SourceFile {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    fn span(&self) -> Span {
        self.syntax().span()
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceFile")
            .field("instr_templates", &self.instr_templates)
            .field("instructions", &self.instructions)
            .field("encodings", &self.encodings)
            .field("asm_decls", &self.asm_decls)
            .finish()
    }
}

impl InstrTemplateDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::InstrTemplateDecl {
            return None;
        }

        let params = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrTemplateParams => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .filter_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrTemplateSingleParam => {
                    InstrTemplateParameterDecl::new(node)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let fields = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::StructBody => Some(node),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .filter_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::StructField => {
                    StructFieldDecl::new(node)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Some(ASTNodeKind::InstrTemplateDecl(Self {
            syntax,
            params,
            fields,
        }))
    }

    pub fn name(&self) -> String {
        self.syntax
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

    pub fn parameters<'a>(&'a self) -> &'a [ASTNodeKind] {
        &self.params
    }

    pub fn fields<'a>(&'a self) -> &'a [ASTNodeKind] {
        &self.fields
    }
}

impl fmt::Debug for InstrTemplateDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateDecl")
            .field("name", &self.name())
            .field("params", &self.parameters())
            .field("fields", &self.fields())
            .finish()
    }
}

impl InstrTemplateParameterDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::InstrTemplateSingleParam {
            return None;
        }

        Some(ASTNodeKind::InstrTemplateParameterDecl(Self { syntax }))
    }

    pub fn name(&self) -> String {
        self.syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node)
                    if node.kind() == SyntaxKind::InstrTemplateSingleParamName =>
                {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .map(|node| node.children())
            .flatten()
            .find_map(|child| match child {
                crate::SyntaxElement::Token(token) if token.kind() == SyntaxKind::Identifier => {
                    Some(token.text().to_string())
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn ty(&self) -> Type {
        let ty = self
            .syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::Type => Some(node),
                _ => None,
            })
            .unwrap();

        Type::new(ty).unwrap()
    }
}

impl fmt::Debug for InstrTemplateParameterDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateParameterDecl")
            .field("name", &self.name())
            .field("type", &self.ty())
            .finish()
    }
}

impl InstrDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::InstrDecl {
            return None;
        }

        let template_args = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .filter_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplateArg => {
                    InstrTemplateArg::new(node)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Some(ASTNodeKind::InstrDecl(Self {
            syntax,
            template_args,
        }))
    }

    pub fn name(&self) -> String {
        self.syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrName => Some(node),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(|c| match c {
                NodeOrToken::Token(token) if token.kind() == SyntaxKind::Identifier => {
                    Some(token.text().to_string())
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn template_name(&self) -> String {
        self.syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(|c| match c {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplateName => {
                    Some(node)
                }
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(|c| match c {
                NodeOrToken::Token(token) if token.kind() == SyntaxKind::Identifier => {
                    Some(token.text().to_string())
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn template_args<'a>(&'a self) -> &'a [ASTNodeKind] {
        &self.template_args
    }
}

impl fmt::Debug for InstrDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrDecl")
            .field("name", &self.name())
            .field("parent_template_name", &self.template_name())
            .field("parent_template_args", &self.template_args())
            .finish()
    }
}

impl InstrTemplateArg {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::InstrParentTemplateArg {
            return None;
        }

        Some(ASTNodeKind::InstrTemplateArg(Self { syntax }))
    }
}

impl fmt::Debug for InstrTemplateArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateArg").finish()
    }
}

impl EncodingDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::EncodingDecl {
            return None;
        }

        Some(ASTNodeKind::EncodingDecl(Self { syntax }))
    }
}

impl fmt::Debug for EncodingDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodingDecl").finish()
    }
}

impl AsmDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::AsmDecl {
            return None;
        }

        Some(ASTNodeKind::AsmDecl(Self { syntax }))
    }
}

impl fmt::Debug for AsmDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsmDecl").finish()
    }
}

impl StructFieldDecl {
    pub fn new(syntax: SyntaxNode) -> Option<ASTNodeKind> {
        if syntax.kind() != SyntaxKind::StructField {
            return None;
        }

        Some(ASTNodeKind::StructFieldDecl(Self { syntax }))
    }

    pub fn name(&self) -> String {
        self.syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Token(t) if t.kind() == SyntaxKind::Identifier => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .unwrap_or("unknown".to_string())
    }

    pub fn ty(&self) -> Type {
        self.syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::Type => Type::new(n),
                _ => None,
            })
            .unwrap()
    }
}

impl fmt::Debug for StructFieldDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructFieldDecl")
            .field("name", &self.name())
            .field("type", &self.ty())
            .finish()
    }
}

impl fmt::Debug for ASTNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNodeKind::AsmDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::EncodingDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::InstrTemplateDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::InstrTemplateParameterDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::InstrDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::SourceFile(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::StructFieldDecl(ref n) => fmt::Debug::fmt(n, f),
            ASTNodeKind::InstrTemplateArg(ref n) => fmt::Debug::fmt(n, f),
            _ => todo!(),
        }
    }
}

// pub type TranslationUnit = AstNode<TranslationUnitKind>;
// pub type InstrTemplateDecl = AstNode<InstrTemplateDeclKind>;
// pub type InstrTemplateParameterDecl = AstNode<InstrTemplateParameterDeclKind>;
// pub type InstrDecl = AstNode<InstrDeclKind>;
// pub type InstrTemplateArg = AstNode<InstrTemplateArgKind>;
// pub type StructFieldDecl = AstNode<StructFieldDeclKind>;
// pub type BlockExpr = AstNode<BlockExprKind>;
// pub type LiteralExpr = AstNode<LiteralExprKind>;
// pub type BinOpExpr = AstNode<BinOpExprKind>;
// pub type EncodingDecl = AstNode<EncodingDeclKind>;
// pub type AsmDecl = AstNode<AsmDeclKind>;
//
// pub struct AstNode<NodeKind: AstNodeKind> {
//     syntax_node: SyntaxNode,
//     _p: PhantomData<NodeKind>,
// }
//
// pub fn build(root: SyntaxNode) -> TranslationUnit {
//     TranslationUnit::new(root)
// }
//
// impl TranslationUnit {
//     pub fn instr_templates<'a>(&'a self) -> impl Iterator<Item = InstrTemplateDecl> + 'a {
//         self.syntax_node.children().filter_map(|child| match child {
//             NodeOrToken::Node(node) => {
//                 if node.kind() == SyntaxKind::InstrTemplateDecl {
//                     Some(InstrTemplateDecl::new(node.clone()))
//                 } else {
//                     None
//                 }
//             }
//             _ => None,
//         })
//     }
//
//     pub fn instructions<'a>(&'a self) -> impl Iterator<Item = InstrDecl> + 'a {
//         self.syntax_node.children().filter_map(|child| match child {
//             NodeOrToken::Node(node) => {
//                 if node.kind() == SyntaxKind::InstrDecl {
//                     Some(InstrDecl::new(node.clone()))
//                 } else {
//                     None
//                 }
//             }
//             _ => None,
//         })
//     }
// }
//
// impl fmt::Debug for TranslationUnit {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "TranslationUnit ")?;
//         f.debug_list()
//             .entries(self.instr_templates())
//             .entries(self.instructions())
//             .finish()
//     }
// }
//
// impl InstrTemplateDecl {
//     pub fn name<'a>(&'a self) -> String {
//         self.syntax_node
//             .children()
//             .find_map(|child| match child {
//                 NodeOrToken::Node(node) => {
//                     if node.kind() == SyntaxKind::InstrTemplateName {
//                         Some(node)
//                     } else {
//                         None
//                     }
//                 }
//                 _ => None,
//             })
//             .iter()
//             .map(|node| node.children())
//             .flatten()
//             .find_map(|child| match child {
//                 crate::SyntaxElement::Token(token) => {
//                     if token.kind() == SyntaxKind::Identifier {
//                         Some(token.text().to_string())
//                     } else {
//                         None
//                     }
//                 }
//                 _ => None,
//             })
//             .unwrap_or("unknown".to_string())
//     }
//
//     pub fn template_parameters<'a>(&'a self) {
//         todo!()
//     }
// }
//
// impl fmt::Debug for InstrTemplateDecl {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("InstrTemplateDecl")
//             .field("name", &self.name())
//             .finish()
//     }
// }
//
// impl InstrDecl {
//     pub fn name<'a>(&'a self) -> String {
//         self.syntax_node
//             .children()
//             .find_map(|child| match child {
//                 NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrName => Some(node),
//                 _ => None,
//             })
//             .iter()
//             .flat_map(|node| node.children())
//             .find_map(|child| match child {
//                 crate::SyntaxElement::Token(token) => {
//                     if token.kind() == SyntaxKind::Identifier {
//                         Some(token.text().to_string())
//                     } else {
//                         None
//                     }
//                 }
//                 _ => None,
//             })
//             .unwrap_or("unknown".to_string())
//     }
//
//     pub fn template_name<'a>(&'a self) -> String {
//         self.syntax_node
//             .children()
//             .find_map(|child| match child {
//                 NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
//                     Some(node)
//                 }
//                 _ => None,
//             })
//             .iter()
//             .flat_map(|node| node.children())
//             .find_map(|child| match child {
//                 SyntaxElement::Node(node) if node.kind() == SyntaxKind::InstrParentTemplateName => {
//                     Some(node)
//                 }
//                 _ => None,
//             })
//             .iter()
//             .flat_map(|node| node.children())
//             .find_map(|child| match child {
//                 SyntaxElement::Token(token) if token.kind() == SyntaxKind::Identifier => {
//                     Some(token.text().to_string())
//                 }
//                 _ => None,
//             })
//             .unwrap_or("unknown".to_string())
//     }
//
//     pub fn template_args<'a>(&'a self) -> impl Iterator<Item = InstrTemplateArg> + 'a {
//         self.syntax_node
//             .children()
//             .filter_map(|child| match child {
//                 NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => {
//                     Some(node)
//                 }
//                 _ => None,
//             })
//             // TODO(alexbatashev): can we get rid of `collect` here?
//             .flat_map(|node| node.children().collect::<Vec<_>>())
//             .filter_map(|child| match child {
//                 NodeOrToken::Node(child_node)
//                     if child_node.kind() == SyntaxKind::InstrParentTemplateArg =>
//                 {
//                     Some(InstrTemplateArg::new(child_node.clone()))
//                 }
//                 _ => None,
//             })
//     }
// }
//
// impl fmt::Debug for InstrDecl {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("InstrDecl")
//             .field("name", &self.name())
//             .field("template_name", &self.template_name())
//             .field("template_args", &self.template_args().collect::<Vec<_>>())
//             .finish()
//     }
// }
//
// impl InstrTemplateArg {
//     pub fn expr<'a>(&'a self) -> LiteralExpr {
//         self.syntax_node
//             .children()
//             .find_map(|child| match child {
//                 NodeOrToken::Node(node) if node.kind() == SyntaxKind::LiteralExpr => {
//                     Some(LiteralExpr::new(node))
//                 }
//                 _ => None,
//             })
//             .unwrap()
//     }
// }
//
// impl fmt::Debug for InstrTemplateArg {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("InstrTemplateArg")
//             .field("expr", &self.expr())
//             .finish()
//     }
// }
//
// impl LiteralExpr {
//     pub fn value<'a>(&'a self) -> String {
//         self.syntax_node
//             .children()
//             .find_map(|child| match child {
//                 NodeOrToken::Token(token)
//                     if token.kind() == SyntaxKind::BitLiteral
//                         || token.kind() == SyntaxKind::IntegerLiteral
//                         || token.kind() == SyntaxKind::StringLiteral =>
//                 {
//                     Some(token.text().to_string())
//                 }
//                 _ => None,
//             })
//             .unwrap()
//     }
// }
//
// impl fmt::Debug for LiteralExpr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("LiteralExpr")
//             .field("value", &self.value())
//             .finish()
//     }
// }
//
// impl<NK: AstNodeKind> AstNode<NK> {
//     pub fn new(syntax_node: SyntaxNode) -> Self {
//         assert!(syntax_node.kind() == NK::get_syntax_kind());
//
//         Self {
//             syntax_node,
//             _p: PhantomData::default(),
//         }
//     }
//
//     pub fn span(&self) -> Span {
//         self.syntax_node.span()
//     }
// }
//
// pub trait AstNodeKind {
//     fn get_syntax_kind() -> SyntaxKind;
// }
//
// pub enum TranslationUnitKind {}
//
// impl AstNodeKind for TranslationUnitKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::TranslationUnit
//     }
// }
//
// pub enum InstrTemplateDeclKind {}
//
// impl AstNodeKind for InstrTemplateDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::InstrTemplateDecl
//     }
// }
//
// pub enum InstrTemplateParameterDeclKind {}
//
// impl AstNodeKind for InstrTemplateParameterDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::InstrTemplateSingleParam
//     }
// }
//
// pub enum StructFieldDeclKind {}
//
// impl AstNodeKind for StructFieldDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::StructField
//     }
// }
//
// pub enum BlockExprKind {}
//
// impl AstNodeKind for BlockExprKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::BlockExpr
//     }
// }
//
// pub enum LiteralExprKind {}
//
// impl AstNodeKind for LiteralExprKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::LiteralExpr
//     }
// }
//
// pub enum BinOpExprKind {}
//
// impl AstNodeKind for BinOpExprKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::BinOpExpr
//     }
// }
//
// pub enum EncodingDeclKind {}
//
// impl AstNodeKind for EncodingDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::EncodingDecl
//     }
// }
//
// pub enum AsmDeclKind {}
//
// impl AstNodeKind for AsmDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::AsmDecl
//     }
// }
//
// pub enum InstrDeclKind {}
//
// impl AstNodeKind for InstrDeclKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::InstrDecl
//     }
// }
//
// pub enum InstrTemplateArgKind {}
//
// impl AstNodeKind for InstrTemplateArgKind {
//     fn get_syntax_kind() -> SyntaxKind {
//         SyntaxKind::InstrParentTemplateArg
//     }
// }
