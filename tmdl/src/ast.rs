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
pub enum Item {
    InstrTemplateDecl(InstrTemplateDecl),
    InstrDecl(InstrDecl),
    EncodingDecl(EncodingDecl),
    AsmDecl(AsmDecl),
    EnumDecl(EnumDecl),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal,
}

#[derive(Clone)]
pub struct SourceFile {
    syntax: SyntaxNode,
    items: Vec<Item>,
}

#[derive(Clone)]
pub struct InstrTemplateDecl {
    syntax: SyntaxNode,
    params: Vec<InstrTemplateParameterDecl>,
    fields: Vec<StructFieldDecl>,
}

#[derive(Clone)]
pub struct InstrTemplateParameterDecl {
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub struct InstrDecl {
    syntax: SyntaxNode,
    template_args: Vec<InstrTemplateArg>,
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

#[derive(Clone)]
pub struct EnumDecl {
    syntax: SyntaxNode,
    variants: Vec<EnumVariantDecl>,
}

#[derive(Clone)]
pub struct EnumVariantDecl {
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

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::InstrTemplateDecl(i) => i.fmt(f),
            Item::InstrDecl(i) => i.fmt(f),
            Item::EncodingDecl(i) => i.fmt(f),
            Item::AsmDecl(i) => i.fmt(f),
            Item::EnumDecl(i) => i.fmt(f),
        }
    }
}

impl From<InstrTemplateDecl> for Item {
    fn from(i: InstrTemplateDecl) -> Self {
        Item::InstrTemplateDecl(i)
    }
}

impl From<InstrDecl> for Item {
    fn from(i: InstrDecl) -> Self {
        Item::InstrDecl(i)
    }
}

impl From<EncodingDecl> for Item {
    fn from(i: EncodingDecl) -> Self {
        Item::EncodingDecl(i)
    }
}

impl From<AsmDecl> for Item {
    fn from(i: AsmDecl) -> Self {
        Item::AsmDecl(i)
    }
}

impl From<EnumDecl> for Item {
    fn from(i: EnumDecl) -> Self {
        Item::EnumDecl(i)
    }
}

impl SourceFile {
    pub fn new(root: SyntaxNode) -> Option<SourceFile> {
        if root.kind() != SyntaxKind::TranslationUnit {
            return None;
        }

        let items = root
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Node(node) => match node.kind() {
                    SyntaxKind::InstrTemplateDecl => {
                        InstrTemplateDecl::new(node.clone()).map(|t| t.into())
                    }
                    SyntaxKind::InstrDecl => InstrDecl::new(node.clone()).map(|t| t.into()),
                    SyntaxKind::EncodingDecl => EncodingDecl::new(node.clone()).map(|t| t.into()),
                    SyntaxKind::AsmDecl => AsmDecl::new(node.clone()).map(|t| t.into()),
                    SyntaxKind::EnumDecl => EnumDecl::new(node.clone()).map(|t| t.into()),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        Some(SourceFile {
            syntax: root,
            items,
        })
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
            .field("items", &self.items)
            .finish()
    }
}

impl InstrTemplateDecl {
    pub fn new(syntax: SyntaxNode) -> Option<InstrTemplateDecl> {
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

        Some(Self {
            syntax,
            params,
            fields,
        })
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

    pub fn parameters(&self) -> &[InstrTemplateParameterDecl] {
        &self.params
    }

    pub fn fields(&self) -> &[StructFieldDecl] {
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
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::InstrTemplateSingleParam {
            return None;
        }

        Some(Self { syntax })
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
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
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

        Some(Self {
            syntax,
            template_args,
        })
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

    pub fn template_args(&self) -> &[InstrTemplateArg] {
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
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::InstrParentTemplateArg {
            return None;
        }

        Some(Self { syntax })
    }
}

impl fmt::Debug for InstrTemplateArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateArg").finish()
    }
}

impl EncodingDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::EncodingDecl {
            return None;
        }

        Some(Self { syntax })
    }
}

impl fmt::Debug for EncodingDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodingDecl").finish()
    }
}

impl AsmDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::AsmDecl {
            return None;
        }

        Some(Self { syntax })
    }
}

impl fmt::Debug for AsmDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsmDecl").finish()
    }
}

impl StructFieldDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::StructField {
            return None;
        }

        Some(Self { syntax })
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

impl EnumDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::EnumDecl {
            return None;
        }

        let variants = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::EnumBody => Some(n),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .filter_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::EnumVariantDecl => {
                    EnumVariantDecl::new(n)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Some(Self { syntax, variants })
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
}

impl fmt::Debug for EnumDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumDecl")
            .field("name", &self.name())
            .field("variants", &self.variants)
            .finish()
    }
}

impl EnumVariantDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::EnumVariantDecl {
            return None;
        }

        Some(Self { syntax })
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
}

impl fmt::Debug for EnumVariantDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumVariantDecl")
            .field("name", &self.name())
            .finish()
    }
}
