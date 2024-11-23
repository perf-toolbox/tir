use core::fmt;

use lpl::{syntax::NodeOrToken, Span};

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

pub trait ASTNode {
    fn syntax(&self) -> &SyntaxNode;
    fn span(&self) -> Span;
}

pub trait ExprNode {
    fn ty(&self) -> &Type;
}

#[derive(Clone)]
pub enum Type {
    Bits(u16),
    String,
    Integer,
    Unresolved(SyntaxElement),
}

#[derive(Clone)]
pub enum Item {
    InstrTemplateDecl(InstrTemplateDecl),
    InstrDecl(InstrDecl),
    EncodingDecl(EncodingDecl),
    AsmDecl(AsmDecl),
    EnumDecl(EnumDecl),
    ImplDecl(ImplDecl),
    FlagDecl(FlagDecl),
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
    parent_template: Option<SyntaxNode>,
    parent_template_args: Vec<InstrTemplateArg>,
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
    body: BlockExpr,
}

#[derive(Clone)]
pub struct AsmDecl {
    #[allow(dead_code)]
    syntax: SyntaxNode,
    body: BlockExpr,
}

#[derive(Clone)]
pub struct ImplDecl {
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

#[derive(Clone)]
pub struct FlagDecl {
    syntax: SyntaxNode,
}

#[derive(Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Block(BlockExpr),
    BinOp(BinOpExpr),
}

#[derive(Clone)]
pub struct LiteralExpr {
    #[allow(dead_code)]
    syntax: SyntaxNode,
    ty: Type,
}

#[derive(Clone)]
pub struct BlockExpr {
    #[allow(dead_code)]
    syntax: SyntaxNode,
    stmts: Vec<Expr>,
    ty: Type,
}

#[derive(Clone, Debug)]
pub enum BinOpKind {
    BitConcat,
}

#[derive(Clone)]
pub struct BinOpExpr {
    #[allow(dead_code)]
    syntax: SyntaxNode,
    kind: BinOpKind,
    left: Box<Expr>,
    right: Box<Expr>,
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
            _ => Some(Type::Unresolved(NodeOrToken::Node(syntax))),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unresolved(_) => write!(f, "<unresolved>"),
            Type::String => write!(f, "str"),
            Type::Bits(num) => write!(f, "bits<{}>", num),
            Type::Integer => write!(f, "int"),
        }
    }
}

impl Item {
    pub fn name(&self) -> String {
        match self {
            Item::InstrDecl(instr) => instr.name(),
            Item::InstrTemplateDecl(instr) => instr.name(),
            Item::EnumDecl(instr) => instr.name(),
            _ => "unknown".to_owned(),
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
            Item::ImplDecl(i) => i.fmt(f),
            Item::FlagDecl(i) => i.fmt(f),
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

impl From<ImplDecl> for Item {
    fn from(i: ImplDecl) -> Self {
        Item::ImplDecl(i)
    }
}

impl From<FlagDecl> for Item {
    fn from(i: FlagDecl) -> Self {
        Item::FlagDecl(i)
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
                    SyntaxKind::ImplDecl => ImplDecl::new(node.clone()).map(|t| t.into()),
                    SyntaxKind::FlagDecl => FlagDecl::new(node.clone()).map(|t| t.into()),
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

    pub fn items(&self) -> &[Item] {
        &self.items
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

        let parent_template = syntax.children().find_map(|c| match c {
            NodeOrToken::Node(node) if node.kind() == SyntaxKind::InstrParentTemplate => Some(node),
            _ => None,
        });
        let parent_template_args = syntax
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
            params,
            fields,
            parent_template,
            parent_template_args,
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

    pub fn has_parent_template(&self) -> bool {
        self.parent_template.is_some()
    }

    pub fn parent_template_name(&self) -> Option<String> {
        self.parent_template
            .iter()
            .flat_map(|c| c.children())
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::InstrParentTemplateName {
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
    }
}

impl fmt::Debug for InstrTemplateDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstrTemplateDecl")
            .field("name", &self.name())
            .field("params", &self.parameters())
            .field("fields", &self.fields())
            .field("parent_template_name", &self.parent_template_name())
            .field("parent_template_args", &self.parent_template_args)
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

        let body = syntax.children().find_map(|c| match c {
            NodeOrToken::Node(n) if n.kind() == SyntaxKind::BlockExpr => BlockExpr::new(n),
            _ => None,
        })?;

        Some(Self { syntax, body })
    }

    pub fn target_name(&self) -> String {
        self.syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::ImplTargetName {
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
}

impl fmt::Debug for EncodingDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodingDecl")
            .field("target_name", &self.target_name())
            .field("body", &self.body)
            .finish()
    }
}

impl AsmDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::AsmDecl {
            return None;
        }

        let body = syntax.children().find_map(|c| match c {
            NodeOrToken::Node(n) if n.kind() == SyntaxKind::BlockExpr => BlockExpr::new(n),
            _ => None,
        })?;

        Some(Self { syntax, body })
    }

    pub fn target_name(&self) -> String {
        self.syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::ImplTargetName {
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
}

impl fmt::Debug for AsmDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsmDecl")
            .field("target_name", &self.target_name())
            .field("body", &self.body)
            .finish()
    }
}

impl ImplDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::ImplDecl {
            return None;
        }

        Some(Self { syntax })
    }

    pub fn target_name(&self) -> String {
        self.syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::ImplTargetName {
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

    pub fn trait_name(&self) -> String {
        self.syntax
            .children()
            .find_map(|child| match child {
                NodeOrToken::Node(node) => {
                    if node.kind() == SyntaxKind::ImplTraitName {
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
}

impl fmt::Debug for ImplDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImplDecl")
            .field("trait_name", &self.trait_name())
            .field("target_name", &self.target_name())
            .finish()
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

    pub fn variants(&self) -> &[EnumVariantDecl] {
        &self.variants
    }

    pub fn doc(&self) -> Option<String> {
        let all: Vec<_> = self
            .syntax
            .children()
            .filter_map(|c| match c {
                NodeOrToken::Token(t) if t.kind() == SyntaxKind::LocalDocComment => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();

        if all.is_empty() {
            None
        } else {
            Some(all.join("\n"))
        }
    }
}

impl fmt::Debug for EnumDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumDecl")
            .field("name", &self.name())
            .field("doc", &self.doc())
            .field("variants", &self.variants())
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

    pub fn doc(&self) -> Option<String> {
        let all: Vec<_> = self
            .syntax
            .children()
            .filter_map(|c| match c {
                NodeOrToken::Token(t) if t.kind() == SyntaxKind::LocalDocComment => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();

        if all.is_empty() {
            None
        } else {
            Some(all.join("\n"))
        }
    }
}

impl fmt::Debug for EnumVariantDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumVariantDecl")
            .field("name", &self.name())
            .field("doc", &self.doc())
            .finish()
    }
}

impl FlagDecl {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::FlagDecl {
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

    pub fn doc(&self) -> Option<String> {
        let all: Vec<_> = self
            .syntax
            .children()
            .filter_map(|c| match c {
                NodeOrToken::Token(t) if t.kind() == SyntaxKind::LocalDocComment => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();

        if all.is_empty() {
            None
        } else {
            Some(all.join("\n"))
        }
    }
}

impl fmt::Debug for FlagDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlagDecl")
            .field("name", &self.name())
            .field("doc", &self.doc())
            .finish()
    }
}

impl From<LiteralExpr> for Expr {
    fn from(value: LiteralExpr) -> Self {
        Expr::Literal(value)
    }
}

impl From<BlockExpr> for Expr {
    fn from(value: BlockExpr) -> Self {
        Expr::Block(value)
    }
}

impl From<BinOpExpr> for Expr {
    fn from(value: BinOpExpr) -> Self {
        Expr::BinOp(value)
    }
}

impl ExprNode for Expr {
    fn ty(&self) -> &Type {
        match self {
            Expr::Literal(l) => l.ty(),
            Expr::Block(b) => b.ty(),
            Expr::BinOp(b) => b.ty(),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(l) => l.fmt(f),
            Expr::Block(b) => b.fmt(f),
            Expr::BinOp(b) => b.fmt(f),
        }
    }
}

impl LiteralExpr {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::LiteralExpr {
            return None;
        }

        let ty = syntax.children().find_map(|c| match c {
            NodeOrToken::Token(token) => {
                if token.kind() == SyntaxKind::IntegerLiteral {
                    Some(Type::Integer)
                } else if token.kind() == SyntaxKind::StringLiteral {
                    Some(Type::String)
                } else if token.kind() == SyntaxKind::BitLiteral {
                    Some(Type::Bits((token.text_len() - 2) as u16))
                } else if token.kind() == SyntaxKind::Identifier {
                    Some(Type::Unresolved(NodeOrToken::Token(token)))
                } else {
                    None
                }
            }
            NodeOrToken::Node(node) if node.kind() == SyntaxKind::StructFieldAccess => {
                Some(Type::Unresolved(NodeOrToken::Node(node)))
            }
            _ => None,
        })?;

        Some(Self { syntax, ty })
    }
}

impl ExprNode for LiteralExpr {
    fn ty(&self) -> &Type {
        &self.ty
    }
}

impl fmt::Debug for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiteralExpr")
            .field("type", &self.ty)
            .finish()
    }
}

impl ExprNode for BlockExpr {
    fn ty(&self) -> &Type {
        &self.ty
    }
}

impl BlockExpr {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::BlockExpr {
            return None;
        }

        let stmts = syntax
            .children()
            .filter_map(map_expr)
            .collect::<Vec<Expr>>();

        let ty = stmts.last().map(|e| e.ty()).cloned()?;

        Some(Self { syntax, stmts, ty })
    }
}

impl fmt::Debug for BlockExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockExpr")
            .field("type", &self.ty)
            .field("stmts", &self.stmts)
            .finish()
    }
}

impl BinOpExpr {
    pub fn new(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() != SyntaxKind::BinOpExpr {
            return None;
        }

        let left: Box<Expr> = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::BinOpExprLeft => Some(n),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(map_expr)
            .map(Box::new)
            .unwrap();
        let kind: BinOpKind = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::BinOpExprOp => Some(n),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(|c| match c {
                NodeOrToken::Token(token) => match token.kind() {
                    SyntaxKind::At => Some(BinOpKind::BitConcat),
                    _ => None,
                },
                _ => None,
            })?;
        let right: Box<Expr> = syntax
            .children()
            .find_map(|c| match c {
                NodeOrToken::Node(n) if n.kind() == SyntaxKind::BinOpExprRight => Some(n),
                _ => None,
            })
            .iter()
            .flat_map(|n| n.children())
            .find_map(map_expr)
            .map(Box::new)?;

        Some(Self {
            syntax,
            kind,
            left,
            right,
        })
    }
}

impl fmt::Debug for BinOpExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinOpExpr")
            .field("kind", &self.kind)
            .field("left", &*self.left)
            .field("right", &*self.right)
            .finish()
    }
}

impl ExprNode for BinOpExpr {
    fn ty(&self) -> &Type {
        self.left.ty()
    }
}

fn map_expr(element: SyntaxElement) -> Option<Expr> {
    match element {
        NodeOrToken::Node(node) => match node.kind() {
            SyntaxKind::LiteralExpr => LiteralExpr::new(node).map(|e| e.into()),
            SyntaxKind::BlockExpr => BlockExpr::new(node).map(|e| e.into()),
            SyntaxKind::BinOpExpr => BinOpExpr::new(node).map(|e| e.into()),
            _ => None,
        },
        _ => None,
    }
}
