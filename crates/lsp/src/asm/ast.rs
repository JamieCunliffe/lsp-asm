use super::registers::registers_for_architecture;
use base::register::{RegisterKind, Registers};
use base::FileType;
use parser::config::ParserConfig;
use symbolic::common::{Name, NameMangling};
use symbolic::demangle::{Demangle, DemangleOptions};
use syntax::ast::{SyntaxKind, SyntaxNode, SyntaxToken};

pub(crate) trait AstToken<'st, 'c> {
    fn cast(token: &'st SyntaxToken, config: &'c ParserConfig) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &'st SyntaxToken;
}

pub struct LabelToken<'st, 'ft> {
    token: &'st SyntaxToken,
    file_type: &'ft FileType,
}

impl<'st, 'ft> AstToken<'st, 'ft> for LabelToken<'st, 'ft> {
    fn cast(token: &'st SyntaxToken, config: &'ft ParserConfig) -> Option<Self> {
        if matches!(token.kind(), SyntaxKind::LABEL | SyntaxKind::LOCAL_LABEL) {
            Some(LabelToken {
                token,
                file_type: &config.file_type,
            })
        } else {
            None
        }
    }

    fn syntax(&self) -> &'st SyntaxToken {
        self.token
    }
}

impl<'st, 'ft> LabelToken<'st, 'ft> {
    pub(crate) fn name(&self) -> &'st str {
        let text = self.token.text().trim_end_matches(':');

        match self.file_type {
            FileType::Assembly => text,
            FileType::ObjDump => text.trim_start_matches('<').trim_end_matches('>'),
        }
    }

    pub(crate) fn demangle(&self) -> Option<(String, String)> {
        let name = Name::new(
            self.name(),
            NameMangling::Mangled,
            symbolic::common::Language::Unknown,
        );

        let lang = name.detect_language();
        name.demangle(DemangleOptions::complete())
            .map(|sym| (sym, lang.to_string()))
    }
}

pub struct NumericToken<'st> {
    token: &'st SyntaxToken,
}
impl<'st, 'c> AstToken<'st, 'c> for NumericToken<'st> {
    fn cast(token: &'st SyntaxToken, _: &'c ParserConfig) -> Option<Self>
    where
        Self: Sized,
    {
        if matches!(token.kind(), SyntaxKind::NUMBER) {
            Some(Self { token })
        } else {
            None
        }
    }

    fn syntax(&self) -> &'st SyntaxToken {
        self.token
    }
}
impl<'st> NumericToken<'st> {
    pub(crate) fn value(&self) -> i128 {
        parser::parse_number(self.token.text()).unwrap()
    }
}

pub struct RegisterToken<'st, 'c> {
    token: &'st SyntaxToken,
    config: &'c ParserConfig,
}
impl<'st, 'c> AstToken<'st, 'c> for RegisterToken<'st, 'c> {
    fn cast(token: &'st SyntaxToken, config: &'c ParserConfig) -> Option<Self>
    where
        Self: Sized,
    {
        matches!(token.kind(), SyntaxKind::REGISTER).then(|| Self { token, config })
    }

    fn syntax(&self) -> &'st SyntaxToken {
        self.token
    }
}
impl<'st, 'c> RegisterToken<'st, 'c> {
    pub(crate) fn register_kind(&self) -> RegisterKind {
        registers_for_architecture(&self.config.architecture)
            .map(|r| r.get_kind(self.token.text()))
            .unwrap_or(RegisterKind::NONE)
    }
}

pub(crate) trait AstNode<'s> {
    fn cast(token: &'s SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &'s SyntaxNode;
}

pub struct LabelNode<'s> {
    syntax: &'s SyntaxNode,
}

impl<'s> AstNode<'s> for LabelNode<'s> {
    fn cast(node: &'s SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        matches!(node.kind(), SyntaxKind::LABEL).then(|| Self { syntax: node })
    }

    fn syntax(&self) -> &'s SyntaxNode {
        self.syntax
    }
}

impl<'s> LabelNode<'s> {
    pub(super) fn sub_labels(&self) -> impl Iterator<Item = SyntaxNode> {
        self.syntax
            .descendants()
            .filter(|d| matches!(d.kind(), SyntaxKind::LOCAL_LABEL))
    }
}

pub struct LocalLabelNode<'s> {
    syntax: &'s SyntaxNode,
}

impl<'s> AstNode<'s> for LocalLabelNode<'s> {
    fn cast(node: &'s SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        matches!(node.kind(), SyntaxKind::LOCAL_LABEL).then(|| Self { syntax: node })
    }

    fn syntax(&self) -> &'s SyntaxNode {
        self.syntax
    }
}
