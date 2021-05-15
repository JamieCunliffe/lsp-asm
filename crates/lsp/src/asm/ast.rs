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
