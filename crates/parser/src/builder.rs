use std::cell::{Ref, RefCell};

use rowan::{GreenNode, GreenToken, NodeOrToken};

use syntax::alias::Alias;
use syntax::ast::SyntaxKind;

pub struct Builder {
    child: RefCell<Vec<NodeOrToken<GreenNode, GreenToken>>>,
    parent: RefCell<Vec<(usize, SyntaxKind)>>,
    pub(crate) alias: RefCell<Alias>,
}

impl Builder {
    pub(super) fn new(size_hint: usize) -> Self {
        Self {
            child: RefCell::new(Vec::with_capacity(size_hint)),
            parent: Default::default(),
            alias: Default::default(),
        }
    }

    pub(super) fn alias(&self) -> Ref<Alias> {
        self.alias.borrow()
    }

    pub(super) fn start_node(&self, kind: SyntaxKind) {
        let pos = self.child.borrow().len();
        self.parent.borrow_mut().push((pos, kind));
    }

    pub(super) fn token(&self, kind: SyntaxKind, text: &str) {
        self.child
            .borrow_mut()
            .push(NodeOrToken::Token(GreenToken::new(kind.into(), text)));
    }

    pub(super) fn finish_node(&self) {
        let mut parent = self.parent.borrow_mut();
        let mut child = self.child.borrow_mut();

        let (start_pos, kind) = parent.pop().unwrap();
        let items = child.drain(start_pos..).collect::<Vec<_>>();
        let node = GreenNode::new(kind.into(), items);

        if kind == SyntaxKind::ALIAS {
            self.alias.borrow_mut().add_alias(&node)
        }

        child.push(NodeOrToken::Node(node));
    }

    pub(super) fn finish(&self) -> GreenNode {
        let n = self.parent.borrow().len();
        for _ in 0..n {
            self.finish_node();
        }

        match self.child.borrow_mut().pop().unwrap() {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("Invalid syntax tree built"),
        }
    }

    pub(super) fn change_node_kind(&self, new_kind: SyntaxKind) {
        let index = self.parent.borrow().len() - 1;
        if let Some(mut i) = self.parent.borrow_mut().get_mut(index) {
            i.1 = new_kind
        }
    }

    pub(super) fn change_previous_token_kind(&self, offset: usize, new_kind: SyntaxKind) {
        let index = self.child.borrow().len() - 1 - offset;
        let text = self
            .child
            .borrow()
            .get(index)
            .map(|t| t.as_token().map(|t| t.text().to_string()))
            .flatten()
            .unwrap_or_default();

        let token = NodeOrToken::Token(GreenToken::new(new_kind.into(), &text));
        let _ = std::mem::replace(&mut self.child.borrow_mut()[index], token);
    }

    pub(super) fn current_indent_is_kind(&self, kind: SyntaxKind) -> bool {
        self.parent
            .borrow()
            .last()
            .map(|(_, k)| k == &kind)
            .unwrap_or(false)
    }
}
