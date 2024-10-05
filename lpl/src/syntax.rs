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

    pub fn span(&self) -> Span {
        self.span.clone()
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

    pub fn replace_child(&self, index: usize, new_child: GreenElement<SK>) -> GreenNode<SK> {
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

#[derive(Clone, Copy, PartialEq)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

impl<N, T> NodeOrToken<N, T> {
    pub fn as_node(&self) -> &N {
        match self {
            NodeOrToken::Node(n) => n,
            _ => panic!("Not a Node"),
        }
    }

    pub fn as_token(&self) -> &T {
        match self {
            NodeOrToken::Token(t) => t,
            _ => panic!("Not a Token"),
        }
    }
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

impl<N, T> fmt::Debug for NodeOrToken<N, T>
where
    N: fmt::Debug,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrToken::Node(node) => fmt::Debug::fmt(node, f),
            NodeOrToken::Token(token) => fmt::Debug::fmt(token, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn green_token_data_new() {
        let kind: u32 = 1;
        let text = String::from("hello");
        let span = Span::empty();
        let token = GreenTokenData::<u32>::new(kind, text.clone()).spanned(span);
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), &text);
    }

    #[test]
    fn green_token_data_span() {
        let kind: u32 = 1;
        let text = String::from("hello");
        let span = Span::empty();
        let token = GreenTokenData::<u32>::new(kind, text.clone()).spanned(span.clone());
        assert_eq!(&token.span, &span);
    }

    #[test]
    fn green_node_data_new() {
        let kind: u32 = 1;
        let children = vec![GreenElement::Node(GreenNodeData::<u32>::new(
            2,
            vec![],
            Span::empty(),
        ))];
        let span = Span::empty();
        let node = GreenNodeData::<u32>::new(kind, children.clone(), span);
        assert_eq!(node.kind(), kind);
        assert_eq!(node.children().len(), 1);
    }

    #[test]
    fn green_node_data_replace_child() {
        let kind: u32 = 1;
        let node = GreenNodeData::<u32>::new(
            kind,
            vec![
                GreenElement::Token(GreenTokenData::<u32>::new(2, "2".to_string())),
                GreenElement::Node(GreenNodeData::<u32>::new(3, vec![], Span::empty())),
            ],
            Span::empty(),
        );
        assert_eq!(node.children().len(), 2);
        let new_child = GreenElement::Node(GreenNodeData::<u32>::new(4, vec![], Span::empty()));
        let node = node.replace_child(1, new_child.clone());
        assert_eq!(node.children().len(), 2);
        assert_eq!(
            &node.children()[0],
            &GreenElement::Token(GreenTokenData::<u32>::new(2, "2".to_string()))
        );
        assert_eq!(&node.children()[1], &new_child);
    }

    #[test]
    fn green_node_data_display() {
        let kind: u32 = 1;
        let node = GreenNodeData::<u32>::new(
            kind,
            vec![
                GreenElement::Token(GreenTokenData::<u32>::new(2, "2".to_string())),
                GreenElement::Node(GreenNodeData::<u32>::new(3, vec![], Span::empty())),
            ],
            Span::empty(),
        );
        assert_eq!(format!("{}", node), "2");
    }

    #[test]
    fn green_token_data_display() {
        let kind: u32 = 1;
        let text = String::from("hello");
        let token = GreenTokenData::<u32>::new(kind, text.clone()).spanned(Span::empty());
        assert_eq!(format!("{}", token), "hello");
    }

    #[test]
    fn node_or_token_display() {
        let token = GreenTokenData::<u32>::new(1, "1".to_string());
        let node = GreenNodeData::<u32>::new(2, vec![], Span::empty());
        assert_eq!(
            format!(
                "{}",
                NodeOrToken::<GreenNode<u32>, GreenToken<u32>>::Node(node)
            ),
            ""
        );
        assert_eq!(
            format!(
                "{}",
                NodeOrToken::<GreenNode<u32>, GreenToken<u32>>::Token(token)
            ),
            "1"
        );
    }
}
