use core::fmt::Debug;
use std::fmt;
use std::iter;
use std::rc::Rc;
use std::sync::Arc;

use crate::combinators::NotTuple;
use crate::Span;

pub type GreenToken<SK> = Arc<GreenTokenData<SK>>;
pub type GreenNode<SK> = Arc<GreenNodeData<SK>>;
pub type GreenElement<SK> = NodeOrToken<GreenNode<SK>, GreenToken<SK>>;
pub type RedToken<SK> = Rc<RedTokenData<SK>>;
pub type RedNode<SK> = Rc<RedNodeData<SK>>;
pub type RedElement<SK> = NodeOrToken<RedNode<SK>, RedToken<SK>>;

pub trait SyntaxLike: Copy + Clone + Debug + PartialEq {
    fn is_trivia(&self) -> bool;
}

impl SyntaxLike for u32 {
    fn is_trivia(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreenTokenData<SK: SyntaxLike> {
    kind: SK,
    text: String,
    span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreenNodeData<SK: SyntaxLike> {
    kind: SK,
    children: Vec<GreenElement<SK>>,
    span: Span,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedTokenData<SK: SyntaxLike> {
    parent: Option<RedNode<SK>>,
    green: GreenToken<SK>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedNodeData<SK: SyntaxLike> {
    parent: Option<RedNode<SK>>,
    green: GreenNode<SK>,
}

impl<SK: SyntaxLike> GreenTokenData<SK> {
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

impl<SK> NotTuple for GreenTokenData<SK> where SK: SyntaxLike {}

impl<SK: SyntaxLike> fmt::Display for GreenTokenData<SK> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl<SK> NotTuple for GreenNodeData<SK> where SK: SyntaxLike {}

impl<SK: SyntaxLike> GreenNodeData<SK> {
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

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn text_len(&self) -> usize {
        0
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

impl<SK: SyntaxLike> fmt::Display for GreenNodeData<SK> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            fmt::Display::fmt(child, f)?;
        }
        Ok(())
    }
}

impl<N, T> NotTuple for NodeOrToken<N, T> {}

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

impl<SK: SyntaxLike> GreenElement<SK> {
    pub fn is_trivia(&self) -> bool {
        match self {
            NodeOrToken::Token(t) => t.kind().is_trivia(),
            _ => false,
        }
    }
}

impl<SK: SyntaxLike> PartialEq<&str> for GreenElement<SK> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            NodeOrToken::Token(t) => &t.text() == other,
            NodeOrToken::Node(_) => todo!("Nodes do not support text interface yet"),
        }
    }
}

impl<SK: SyntaxLike> RedTokenData<SK> {
    pub fn new(green: GreenToken<SK>) -> RedToken<SK> {
        Rc::new(RedTokenData {
            parent: None,
            green,
        })
    }

    fn green(&self) -> &GreenToken<SK> {
        &self.green
    }

    pub fn kind(&self) -> SK {
        self.green().kind()
    }

    pub fn text_len(&self) -> usize {
        self.green().text_len()
    }

    pub fn text(&self) -> &str {
        self.green().text()
    }

    pub fn span(&self) -> Span {
        self.green().span()
    }

    pub fn parent(&self) -> Option<&RedNode<SK>> {
        self.parent.as_ref()
    }
}

impl<SK: SyntaxLike> RedNodeData<SK> {
    pub fn new(green: GreenNode<SK>) -> RedNode<SK> {
        Rc::new(RedNodeData {
            parent: None,
            green,
        })
    }

    fn green(&self) -> &GreenNode<SK> {
        &self.green
    }

    pub fn kind(&self) -> SK {
        self.green().kind()
    }

    pub fn text_len(&self) -> usize {
        self.green().text_len()
    }

    pub fn parent(&self) -> Option<&RedNode<SK>> {
        self.parent.as_ref()
    }

    pub fn span(&self) -> Span {
        self.green().span()
    }

    pub fn children<'a>(self: &'a RedNode<SK>) -> impl Iterator<Item = RedElement<SK>> + 'a {
        self.green().children().iter().map(|c| match c {
            NodeOrToken::Node(n) => NodeOrToken::Node(Rc::new(RedNodeData {
                parent: Some(self.clone()),
                green: Arc::clone(n),
            })),
            NodeOrToken::Token(t) => NodeOrToken::Token(Rc::new(RedTokenData {
                parent: Some(self.clone()),
                green: Arc::clone(t),
            })),
        })
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
