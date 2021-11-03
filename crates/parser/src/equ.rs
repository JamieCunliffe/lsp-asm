use std::mem;

use rowan::{GreenNode, GreenToken, NodeOrToken};
use syntax::ast::SyntaxKind;

pub(crate) fn transform_equ_node(items: &mut Vec<NodeOrToken<GreenNode, GreenToken>>) {
    let name_element = items
        .iter_mut()
        .skip_while(|i| i.kind() != SyntaxKind::WHITESPACE.into())
        .nth(1);

    if let Some(name) = name_element.as_ref().map(|t| t.as_token()).flatten() {
        let mut token = NodeOrToken::Token(GreenToken::new(SyntaxKind::NAME.into(), name.text()));
        mem::swap(&mut token, name_element.unwrap());
    }

    let expr = items
        .iter()
        .enumerate()
        .skip_while(|(_, i)| i.kind() != SyntaxKind::COMMA.into())
        .nth(1);

    if let Some((index, _)) = expr {
        let expr = items.drain(index..).collect::<Vec<_>>();
        let node = GreenNode::new(SyntaxKind::EXPR.into(), expr);
        items.push(NodeOrToken::Node(node));
    }
}
