use crate::config::LSPConfig;
use crate::types::{Architecture, DocumentPosition, DocumentRange, LineNumber};

use super::ast::{AstToken, LabelToken, SyntaxKind, SyntaxNode, SyntaxToken};
use super::config::{FileType, ParserConfig};
use super::debug::DebugMap;

use byte_unit::Byte;
use once_cell::sync::OnceCell;
use rayon::prelude::*;

use rowan::{TextRange, TextSize};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    root: SyntaxNode,
    filesize: Byte,
    config: ParserConfig,
    line_index: PositionInfo,
    debug_map: OnceCell<DebugMap>,
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
    pub fn from(data: &str, config: &LSPConfig) -> Self {
        let filesize = Byte::from_bytes(data.len() as _);
        let mut config = ParserConfig::new(&Self::determine_architecture(data, config));
        config.file_type = Self::determine_filetype(data);

        let root = Self::parse_asm(data, &config);

        Self {
            line_index: PositionInfo::new(&data),
            filesize,
            root,
            config,
            debug_map: OnceCell::new(),
        }
    }

    pub(crate) fn tree(&self) -> &SyntaxNode {
        &self.root
    }

    pub(crate) fn filesize(&self) -> Byte {
        self.filesize
    }

    pub(super) fn debug_map(&self) -> &DebugMap {
        &self.debug_map.get_or_init(|| DebugMap::new(self.tree()))
    }

    pub(crate) fn token<'st, 'c, T>(&'c self, token: &'st SyntaxToken) -> Option<T>
    where
        T: AstToken<'st, 'c>,
    {
        T::cast(token, &self.config)
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
    pub(crate) fn tokens_in_range<'a>(&self, range: &'a TextRange) -> impl Iterator<Item = SyntaxToken> + 'a {
        self.root
            .descendants_with_tokens()
            .filter_map(|t| t.into_token())
            .skip_while(move |token| !range.contains_inclusive(token.text_range().start()))
            .take_while(move |token| range.contains_inclusive(token.text_range().start()))
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
            SyntaxKind::LABEL => self
                .token::<LabelToken>(token)
                .map(|t| SemanticEq::String(t.name())),
            SyntaxKind::L_PAREN
            | SyntaxKind::R_PAREN
            | SyntaxKind::L_SQ
            | SyntaxKind::R_SQ
            | SyntaxKind::L_CURLY
            | SyntaxKind::R_CURLY
            | SyntaxKind::L_ANGLE
            | SyntaxKind::R_ANGLE
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
            | SyntaxKind::METADATA
            | SyntaxKind::ROOT => None,
        }
    }

    /// Helper function to perform the parsing of data
    fn parse_asm(data: &str, config: &ParserConfig) -> SyntaxNode {
        let nodes = super::combinators::parse(data, config);
        SyntaxNode::new_root(nodes)
    }

    fn determine_filetype(filedata: &str) -> super::config::FileType {
        use regex::Regex;

        lazy_static! {
            static ref OBJDUMP_DETECTION: Regex =
                Regex::new(r#".*:[\t ]+file format .*\n\n\nDisassembly of section .*:"#).unwrap();
        }

        if OBJDUMP_DETECTION.is_match(filedata) {
            FileType::ObjDump
        } else {
            Default::default()
        }
    }

    /// Attempt to determine the architecture that the assembly data is for.
    fn determine_architecture(filedata: &str, config: &LSPConfig) -> Architecture {
        use regex::Regex;
        lazy_static! {
            static ref ARCH_DETECTION: [Regex; 4] = [
                Regex::new(r#"lsp-asm-architecture: (.*) ?"#).unwrap(),
                Regex::new(r#"^\s*\.arch (.*)"#).unwrap(),
                Regex::new(r#".*:[\t ]+file format elf64-(.*)"#).unwrap(),
                Regex::new(r#".*:[\t ]+file format Mach-O (.*)"#).unwrap(),
            ];
        }
        let arch = ARCH_DETECTION
            .par_iter()
            .filter_map(|regex| regex.captures(filedata))
            .find_map_first(|captures| {
                captures
                    .get(1)
                    .map(|arch| Architecture::from(arch.as_str()))
            })
            .unwrap_or(config.architecture);

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
    pub fn new(data: &str) -> Self {
        let lines = Self::build_lines(data);
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
        let line = self.lines.partition_point(|l| l <= ts) - 1;
        let pos = self.lines.get(line)?;

        let column = ts.checked_sub(*pos).map(|c| c.into()).unwrap_or(0);
        Some(DocumentPosition {
            line: line as _,
            column,
        })
    }

    /// Build a vector with an item for each line in in the data, the item will
    /// contain the `TextSize` for the start of the line.
    fn build_lines(data: &str) -> Vec<TextSize> {
        std::iter::once(TextSize::default())
            .chain(data.char_indices().filter_map(|(pos, c)| {
                if c == '\n' {
                    Some(((pos + 1) as u32).into())
                } else {
                    None
                }
            }))
            .collect()
    }
}
