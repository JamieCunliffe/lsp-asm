use std::fmt::Debug;
use std::iter;

use symbolic::common::{Name, NameMangling};
use symbolic::demangle::{Demangle, DemangleOptions};

use super::config::{FileType, ParserConfig};
use super::registers::{registers_for_architecture, RegisterKind, Registers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
pub(crate) enum SyntaxKind {
    L_PAREN = 0, // '('
    R_PAREN,     // ')'
    L_SQ,
    R_SQ,
    L_CURLY,
    R_CURLY,
    L_ANGLE,
    R_ANGLE,

    MNEMONIC,
    REGISTER,
    TOKEN,
    NUMBER,
    WHITESPACE,
    COMMA,
    OPERATOR,
    STRING,
    LABEL,
    LOCAL_LABEL,
    COMMENT,

    INSTRUCTION,
    DIRECTIVE,
    BRACKETS,

    METADATA,
    // ROOT should be the last element
    ROOT,
}

/// Some boilerplate is needed, as rowan settled on using its own
/// `struct SyntaxKind(u16)` internally, instead of accepting the
/// user's `enum SyntaxKind` as a type parameter.
///
/// First, to easily pass the enum variants into rowan via `.into()`:
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

pub(crate) fn find_parent(token: &SyntaxToken, syntax_kind: SyntaxKind) -> Option<SyntaxNode> {
    iter::successors(Some(token.parent()), |parent| parent.parent())
        .find(|e| e.kind() == syntax_kind)
}

pub(crate) fn find_kind_index(
    node: &SyntaxNode,
    index: i32,
    syntax_kind: SyntaxKind,
) -> Option<SyntaxElement> {
    node.descendants_with_tokens()
        .filter(|d| d.kind() == syntax_kind)
        .nth(index as _)
}

/// Second, implementing the `Language` trait teaches rowan to convert between
/// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
/// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum AssemblyLanguage {}
impl rowan::Language for AssemblyLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub(crate) type SyntaxNode = rowan::SyntaxNode<AssemblyLanguage>;
#[allow(unused)]
pub(crate) type SyntaxToken = rowan::SyntaxToken<AssemblyLanguage>;
#[allow(unused)]
pub(crate) type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

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
        &self.token
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
        super::combinators::parse_number(self.token.text()).unwrap()
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
            .map(|r| r.get_kind(self.token))
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
