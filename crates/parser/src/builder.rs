use std::cell::RefCell;

use rowan::{GreenNode, GreenToken, NodeOrToken};

use syntax::ast::SyntaxKind;

pub struct Builder {
    child: RefCell<Vec<NodeOrToken<GreenNode, GreenToken>>>,
    parent: RefCell<Vec<(usize, SyntaxKind)>>,
}

impl Builder {
    pub(super) fn new(size_hint: usize) -> Self {
        Self {
            child: RefCell::new(Vec::with_capacity(size_hint)),
            parent: Default::default(),
        }
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
        child.push(NodeOrToken::Node(GreenNode::new(kind.into(), items)));
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

    pub(super) fn current_indent_is_kind(&self, kind: SyntaxKind) -> bool {
        self.parent
            .borrow()
            .last()
            .map(|(_, k)| k == &kind)
            .unwrap_or(false)
    }
}
