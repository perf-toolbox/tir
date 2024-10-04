use core::fmt::Debug;
use std::fmt;
use std::iter;
use std::sync::Arc;

use crate::Span;

pub type GreenToken<SK> = Arc<GreenTokenData<SK>>;
pub type GreenNode<SK> = Arc<GreenNodeData<SK>>;
pub type GreenElement<SK> = NodeOrToken<GreenNode<SK>, GreenToken<SK>>;

#[derive(Debug, Clone, PartialEq)]
pub struct GreenTokenData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    kind: SK,
    text: String,
    span: Span,
}

impl<SK> GreenTokenData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    pub fn new(kind: SK, text: String) -> GreenToken<SK> {
        Arc::new(GreenTokenData {
            kind,
            text,
            span: Span::empty(),
        })
    }

    pub fn spanned(self: Arc<Self>, span: Span) -> Arc<Self> {
        Arc::new(Self {
            kind: self.kind,
            text: self.text.clone(),
            span,
        })
    }

    pub fn kind(&self) -> SK {
        self.kind
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_len(&self) -> usize {
        self.text.len()
    }
}

impl<SK> fmt::Display for GreenTokenData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreenNodeData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    kind: SK,
    children: Vec<GreenElement<SK>>,
    span: Span,
}

impl<SK> GreenNodeData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    pub fn new(kind: SK, children: Vec<GreenElement<SK>>, span: Span) -> GreenNode<SK> {
        Arc::new(GreenNodeData {
            kind,
            children,
            span,
        })
    }

    pub fn kind(&self) -> SK {
        self.kind
    }

    pub fn children(&self) -> &[GreenElement<SK>] {
        &self.children
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn replace_child(&mut self, index: usize, new_child: GreenElement<SK>) -> GreenNode<SK> {
        let new_children = self
            .children
            .iter()
            .take(index)
            .cloned()
            .chain(iter::once(new_child))
            .chain(self.children.iter().skip(index + 1).cloned())
            .collect();
        GreenNodeData::new(self.kind, new_children, self.span.clone())
    }
}

impl<SK> fmt::Display for GreenNodeData<SK>
where
    SK: Copy + Clone + Debug + PartialEq,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            fmt::Display::fmt(child, f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

impl<N, T> fmt::Display for NodeOrToken<N, T>
where
    N: fmt::Display,
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrToken::Node(node) => fmt::Display::fmt(node, f),
            NodeOrToken::Token(token) => fmt::Display::fmt(token, f),
        }
    }
}
