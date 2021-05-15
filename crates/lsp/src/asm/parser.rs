use crate::types::{Architecture, DocumentPosition, DocumentRange, LineNumber};

use super::ast::{SyntaxKind, SyntaxNode, SyntaxToken};
use super::config::ParserConfig;
use rowan::{GreenNode, GreenToken, NodeOrToken, TextRange, TextSize};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    root: SyntaxNode,
    config: ParserConfig,
    line_index: PositionInfo,
}

/// Helper enum for determining if tokens should be considered equal
#[derive(PartialEq)]
enum SemanticEq<'a> {
    String(&'a str),
    /// A register, the ID in this is only valid for a specific `ParserConfig`
    /// (in reality this means `Architecture` though)
    Register(i8),
    /// The token is a numeric value
    Numeric(i128),
}

impl Parser {
    /// Create a parser from the given data.
    /// * data: The assembly listing to parse
    pub fn from(data: &str) -> Self {
        let config = ParserConfig::new(&Self::determine_architecture(data));
        let root = Self::parse_asm(data, &config);
        info!("Syntax tree: {:#?}", root);
        Self {
            line_index: PositionInfo::new(&root),
            root,
            config,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_test(data: &str, config: ParserConfig) -> Self {
        let root = Self::parse_asm(data, &config);
        Self {
            line_index: PositionInfo::new(&root),
            root,
            config,
        }
    }

    pub(crate) fn tree(&self) -> &SyntaxNode {
        &self.root
    }

    /// Gets the positional information that can be used with tokens.
    pub(crate) fn position(&self) -> &PositionInfo {
        &self.line_index
    }

    /// Gets the the token at the given `position`.
    pub(crate) fn token_at_point(&self, position: &DocumentPosition) -> Option<SyntaxToken> {
        let position = self.position().point_for_position(position)?;
        self.tree()
            .descendants_with_tokens()
            .filter_map(|t| t.into_token())
            .find(|token| token.text_range().contains(position))
    }

    /// Gets the tokens that are contained within `range`.
    pub(crate) fn tokens_in_range(&self, range: &TextRange) -> Vec<SyntaxToken> {
        self.root
            .descendants_with_tokens()
            .filter_map(|t| t.into_token())
            .filter(|t| range.contains_inclusive(t.text_range().start()))
            .collect()
    }

    /// Checks to see if the two tokens are refering to the same thing, for instance,
    /// `label1:` and `label1` are considered the same thing, and registers that are
    /// the same register in hardware will be considered the same.
    pub(crate) fn token_text_equal(&self, lhs: &SyntaxToken, rhs: &SyntaxToken) -> bool {
        let lhs = self.token_value(lhs);
        let rhs = self.token_value(rhs);
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            lhs == rhs
        } else {
            false
        }
    }

    /// Gets a value that can be used to determine if two tokens refer to the
    /// same thing e.g. `0x10` and `16` should be considered equal just as
    /// `label1:` and `label` are equal.
    fn token_value<'a>(&self, token: &'a SyntaxToken) -> Option<SemanticEq<'a>> {
        match token.kind() {
            SyntaxKind::REGISTER => Some(SemanticEq::Register(super::registers::register_id(
                token.text(),
                &self.config,
            )?)),
            SyntaxKind::TOKEN => Some(SemanticEq::String(token.text())),
            SyntaxKind::NUMBER => Some(SemanticEq::Numeric(token.text().parse::<i128>().ok()?)),
            SyntaxKind::LABEL => Some(SemanticEq::String(token.text().strip_suffix(':')?)),
            SyntaxKind::L_PAREN
            | SyntaxKind::R_PAREN
            | SyntaxKind::L_SQ
            | SyntaxKind::R_SQ
            | SyntaxKind::L_CURLY
            | SyntaxKind::R_CURLY
            | SyntaxKind::MNEMONIC
            | SyntaxKind::WHITESPACE
            | SyntaxKind::COMMA
            | SyntaxKind::OPERATOR
            | SyntaxKind::STRING
            | SyntaxKind::LOCAL_LABEL
            | SyntaxKind::COMMENT
            | SyntaxKind::INSTRUCTION
            | SyntaxKind::DIRECTIVE
            | SyntaxKind::BRACKETS
            | SyntaxKind::ROOT => None,
        }
    }

    /// Helper function to perform the parsing of data
    fn parse_asm(data: &str, config: &ParserConfig) -> SyntaxNode {
        let nodes = super::combinators::parse(data, config);
        let nodes = Self::indent_labels(nodes, SyntaxKind::LABEL);
        let root = GreenNode::new(SyntaxKind::ROOT.into(), nodes);
        let root = SyntaxNode::new_root(root);
        debug!("Data: {:#?}", root);
        root
    }

    /// Adjusts the syntax tree to indent the labels and the local labels
    fn indent_labels(
        tokens: Vec<NodeOrToken<GreenNode, GreenToken>>,
        kind: SyntaxKind,
    ) -> Vec<NodeOrToken<GreenNode, GreenToken>> {
        let starts_with = tokens
            .first()
            .map(|t| t.as_node())
            .flatten()
            .map(|n| n.kind() == kind.into())
            .unwrap_or(false);

        // Partition the tokens by the labels, these will be ziped to create a (vec<tokens>, label) for each token within the found labels
        let partition = tokens.split(|t| t.kind() == kind.into()).skip(1);
        let labels = tokens.iter().filter(|t| t.kind() == kind.into());
        let mut partition = partition.zip(labels).collect::<Vec<_>>();

        // The partition will be empty if no labels are contained within the syntax tree so just exit out with the original.
        if partition.is_empty() {
            return tokens;
        }

        // Create the label nodes with the child tokens
        let mut indented_tokens = partition
            .drain(..)
            .map(|(tokens, label)| {
                let mut tmp = label
                    .as_node()
                    .unwrap()
                    .children()
                    .map(|c| match c {
                        NodeOrToken::Node(n) => NodeOrToken::Node(n.clone()),
                        NodeOrToken::Token(t) => NodeOrToken::Token(t.clone()),
                    })
                    .collect::<Vec<_>>();
                let mut tokens = Self::indent_labels(tokens.into(), SyntaxKind::LOCAL_LABEL);
                tmp.append(&mut tokens);
                NodeOrToken::Node(GreenNode::new(kind.into(), tmp))
            })
            .collect::<Vec<_>>();

        // If the first node was a label then the partition will have been correct, if it didn't
        // then the partition won't include the elements before the label so we have to add them
        // back in
        if starts_with {
            indented_tokens
        } else {
            let pre_tokens = tokens.split(|t| t.kind() == kind.into()).next().unwrap();
            let mut pre_tokens: Vec<NodeOrToken<GreenNode, GreenToken>> = pre_tokens.into();
            pre_tokens
                .drain(..)
                .chain(indented_tokens.drain(..))
                .collect()
        }
    }

    /// Attempt to determine the architecture that the assembly data is for.
    fn determine_architecture(filedata: &str) -> Architecture {
        use regex::Regex;
        lazy_static! {
            static ref ARCH_DETECTION: [Regex; 2] = [
                Regex::new(r#"lsp-asm-architecture: (.*) ?"#).unwrap(),
                Regex::new(r#"^\s*\.arch (.*)"#).unwrap(),
            ];
        }
        let arch = *ARCH_DETECTION
            .iter()
            .filter_map(|regex| regex.captures(filedata))
            .filter_map(|captures| match captures.get(1) {
                Some(arch) => Some(Architecture::from(arch.as_str())),
                None => None,
            })
            .collect::<Vec<_>>()
            .first()
            .unwrap_or(&Architecture::Unknown);

        debug!("Architecture detected: {:?}", arch);

        match arch {
            Architecture::Unknown => Architecture::from(std::env::consts::ARCH),
            a => a,
        }
    }
}

/// Provides a method for converting `TextSize` data into document line and
/// column numbers.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PositionInfo {
    lines: Vec<TextSize>,
}

impl PositionInfo {
    /// Construct a position info structure from a `SyntaxNode`
    pub fn new(data: &SyntaxNode) -> Self {
        let lines = Self::build_lines(data);
        info!("Build line map: {:#?}", lines);
        Self { lines }
    }

    /// Gets a document position for the given `token`
    pub fn get_position(&self, token: &SyntaxToken) -> Option<DocumentPosition> {
        self.get_position_for_size(&token.text_range().start())
    }

    /// Gets a `DocumentRange` for the given `token`
    pub fn range_for_token(&self, token: &SyntaxToken) -> Option<DocumentRange> {
        let start = self.get_position_for_size(&token.text_range().start())?;
        let end = self.get_position_for_size(&token.text_range().end())?;

        Some(DocumentRange { start, end })
    }

    //// Gets a `DocumentRange` for the given `node`
    pub fn range_for_node(&self, node: &SyntaxNode) -> Option<DocumentRange> {
        let start = self.get_position_for_size(&node.text_range().start())?;
        let end = self.get_position_for_size(&node.text_range().end())?;

        Some(DocumentRange { start, end })
    }

    /// Converts a `DocumentPosition` into a `TextSize` position
    pub fn point_for_position(&self, position: &DocumentPosition) -> Option<TextSize> {
        let line = self.lines.get(position.line as usize)?;
        line.checked_add(position.column.into())
    }

    /// Makes a text range for the given start and end line.
    /// If the requested start or end fall outside the range of the document then
    /// the document start or end will be returned
    pub fn make_range_for_lines(&self, start: LineNumber, end: LineNumber) -> TextRange {
        // Unwrap on first or last is fine as we always insert one line
        let start = self
            .lines
            .get(start as usize)
            .unwrap_or_else(|| self.lines.first().unwrap());
        let end = self
            .lines
            .get(end as usize)
            .unwrap_or_else(|| self.lines.last().unwrap());

        TextRange::new(*start, *end)
    }

    /// Helper function to get the line and column for a text size
    fn get_position_for_size(&self, ts: &TextSize) -> Option<DocumentPosition> {
        let (line, pos) = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, pos)| ts >= (*pos))
            .last()?;

        let column = ts.checked_sub(*pos).map(|c| c.into()).unwrap_or(0);
        Some(DocumentPosition {
            line: line as _,
            column,
        })
    }

    /// Build a vector with an item for each line in in the data, the item will
    /// contain the `TextSize` for the start of the line.
    fn build_lines(data: &SyntaxNode) -> Vec<TextSize> {
        std::iter::once(TextSize::default())
            .chain(
                data.descendants_with_tokens()
                    .filter_map(|x| x.into_token())
                    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
                    .flat_map(|token| {
                        token
                            .text()
                            .char_indices()
                            .filter_map(|(pos, c)| {
                                if c == '\n' {
                                    token
                                        .text_range()
                                        .start()
                                        .checked_add(1.into())?
                                        .checked_add(TextSize::from(pos as u32))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    }),
            )
            .collect::<Vec<_>>()
    }
}
