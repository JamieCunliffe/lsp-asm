use rowan::{GreenNode, GreenToken, NodeOrToken};

use crate::ast::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub enum Position {
    Replace(SyntaxToken),
    After(SyntaxToken),
    Before(SyntaxToken),
    FirstChild(SyntaxNode),
}

pub fn create_token(kind: SyntaxKind, data: &str) -> SyntaxToken {
    SyntaxNode::new_root(GreenNode::new(
        SyntaxKind::ROOT.into(),
        std::iter::once(NodeOrToken::Token(GreenToken::new(kind.into(), data))),
    ))
    .clone_for_update()
    .children_with_tokens()
    .next()
    .and_then(|t| t.into_token())
    .unwrap()
}

pub fn perform_replacements(replacements: Vec<(Position, SyntaxToken)>) {
    replacements
        .into_iter()
        .rev()
        .for_each(|(position, token)| match position {
            Position::Replace(child) => {
                if matches!(token.kind(), SyntaxKind::ROOT) {
                    replace_token(&child.parent().unwrap(), child.index(), vec![])
                } else {
                    replace_token(
                        &child.parent().unwrap(),
                        child.index(),
                        vec![SyntaxElement::Token(token)],
                    )
                }
            }
            Position::After(child) => {
                insert_token(&child.parent().unwrap(), child.index() + 1, token);
            }
            Position::Before(child) => {
                insert_token(&child.parent().unwrap(), child.index(), token);
            }
            Position::FirstChild(node) => {
                node.splice_children(0..0, vec![SyntaxElement::Token(token)])
            }
        });
}

fn insert_token(root: &SyntaxNode, index: usize, token: SyntaxToken) {
    let range = index..index;
    let elem = SyntaxElement::Token(token);
    root.splice_children(range, vec![elem]);
}

pub fn replace_token(root: &SyntaxNode, index: usize, tokens: Vec<SyntaxElement>) {
    let range = index..index + 1;
    root.splice_children(range, tokens);
}
