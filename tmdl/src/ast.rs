use core::fmt;

use lpl::{syntax::NodeOrToken, Span};

use crate::{SyntaxKind, SyntaxNode};

pub trait ASTNode {
    fn syntax(&self) -> &SyntaxNode;
    fn span(&self) -> Span;
}

#[derive(Clone)]
pub enum Type {
    Bits(u16),
    String,
    Unresolved(SyntaxNode),
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

                let num_bits = param.parse::<u16>().ok()?;

                Some(Type::Bits(num_bits))
            }
            _ => Some(Type::Unresolved(syntax)),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unresolved(_) => write!(f, "<unresolved>"),
            Type::String => write!(f, "str"),
            Type::Bits(num) => write!(f, "bits<{}>", num),
        }
    }
}

impl SourceFile {
    pub fn cast(root: SyntaxNode) -> Option<ASTNodeKind> {
        if root.kind() != SyntaxKind::TranslationUnit {
            return None;
        }

        let instr_templates = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrTemplateDecl => {
                    InstrTemplateDecl::cast(node.clone())
                }
                _ => None,
            })
            .collect();

        let instructions = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrDecl => {
                    InstrDecl::cast(node.clone())
                }
                _ => None,
            })
            .collect();

        let encodings = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::EncodingDecl => {
                    EncodingDecl::cast(node.clone())
                }
                _ => None,
            })
            .collect();

        let asm_decls = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) if node.kind() == SyntaxKind::AsmDecl => {
                    AsmDecl::cast(node.clone())
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
                    InstrTemplateParameterDecl::cast(node)
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
                    StructFieldDecl::cast(node)
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

    pub fn parameters(&self) -> &[ASTNodeKind] {
        &self.params
    }

    pub fn fields(&self) -> &[ASTNodeKind] {
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
            .flat_map(|node| node.children())
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
                    InstrTemplateArg::cast(node)
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

    pub fn template_args(&self) -> &[ASTNodeKind] {
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
    pub fn cast(syntax: SyntaxNode) -> Option<ASTNodeKind> {
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
