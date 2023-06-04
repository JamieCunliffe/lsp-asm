use std::fs::read_to_string;

use super::ast::{AstToken, LabelToken};
use super::debug::DebugMap;
use crate::config::LSPConfig;
use crate::file_util::make_file_relative;
use crate::types::{DocumentPosition, DocumentRange, LineNumber};
use base::{Architecture, FileType};
use byte_unit::Byte;
use lsp_types::Url;
use once_cell::sync::{Lazy, OnceCell};
use parser::config::ParserConfig;
use parser::{ParsedData, ParsedInclude};
use rayon::prelude::*;
use rowan::{GreenNode, TextRange, TextSize};
use syntax::alias::Alias;
use syntax::ast::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub(super) id: Url,
    root: GreenNode,
    filesize: Byte,
    config: ParserConfig,
    line_index: PositionInfo,
    debug_map: OnceCell<DebugMap>,
    alias: Alias,
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
    Float(f64),
}

impl Parser {
    /// Create a parser from the given data.
    /// * data: The assembly listing to parse
    pub fn from(uri: Url, data: &str, config: &LSPConfig) -> (Self, Vec<ParsedInclude>) {
        let filesize = Byte::from_bytes(data.len() as _);
        let mut config = Self::config_from_arch(&Self::determine_architecture(data, config));
        config.file_type = FileType::from_contents(data);

        let ParsedData {
            root,
            alias,
            included_files,
            ..
        } = parser::parse_asm(data, &config, Some(uri.as_str()), Self::handle_include);

        (
            Self {
                id: uri,
                line_index: PositionInfo::new(data),
                filesize,
                root,
                config,
                debug_map: OnceCell::new(),
                alias,
            },
            included_files,
        )
    }

    fn handle_include(config: &ParserConfig, from: &str, file: &str) -> Option<ParsedInclude> {
        let included_file = make_file_relative(from, file)?;
        let file_uri = Url::from_file_path(&included_file).ok()?;
        let data = read_to_string(included_file).ok()?;

        let ParsedData {
            root,
            alias,
            included_files,
            ..
        } = parser::parse_asm(&data, config, Some(from), Self::handle_include);

        Some(ParsedInclude {
            alias,
            root,
            included_files,
            id: file_uri.to_string(),
            data,
        })
    }

    pub(crate) fn tree(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.root.clone())
    }

    pub(crate) fn filesize(&self) -> Byte {
        self.filesize
    }

    pub(crate) fn architecture(&self) -> &Architecture {
        &self.config.architecture
    }

    pub(super) fn debug_map(&self) -> &DebugMap {
        self.debug_map.get_or_init(|| DebugMap::new(&self.tree()))
    }

    pub fn alias(&self) -> &Alias {
        &self.alias
    }

    pub fn comment_start(&self) -> &String {
        &self.config.comment_start
    }

    pub fn uri(&self) -> &Url {
        &self.id
    }

    pub fn file_type(&self) -> &FileType {
        &self.config.file_type
    }

    pub(super) fn reconstruct_file(&self) -> String {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.tree().text_range().end().into());
        for token in self.tree().descendants_with_tokens() {
            if let Some(token) = token.as_token() {
                buffer.extend_from_slice(token.text().as_bytes());
            }
        }

        // The data that we have copied into here has come from a &str which
        // is always valid UTF8, therefore it should be perfectly fine to use
        // the unchecked variant of the function, as validation isn't required.
        unsafe { String::from_utf8_unchecked(buffer) }
    }

    pub(super) fn reconstruct_from_tokens(
        &self,
        tokens: impl Iterator<Item = SyntaxToken>,
        range: &TextRange,
    ) -> String {
        let mut buffer: Vec<u8> = Vec::with_capacity(range.end().into());

        for token in tokens {
            buffer.extend_from_slice(token.text().as_bytes());
        }

        // The data that we have copied into here has come from a &str which
        // is always valid UTF8, therefore it should be perfectly fine to use
        // the unchecked variant of the function, as validation isn't required.
        unsafe { String::from_utf8_unchecked(buffer) }
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
        self.tree().token_at_offset(position).right_biased()
    }

    /// Gets the tokens that are contained within `range`.
    pub(crate) fn tokens_in_range(&self, range: TextRange) -> impl Iterator<Item = SyntaxToken> {
        self.tree()
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
    /// `label1:` and `label1` are equal.
    fn token_value<'a>(&self, token: &'a SyntaxToken) -> Option<SemanticEq<'a>> {
        match token.kind() {
            SyntaxKind::REGISTER => Some(SemanticEq::Register(arch::registers::register_id(
                token.text(),
                &self.config,
            )?)),
            SyntaxKind::REGISTER_ALIAS => {
                let register = self.alias.get_register_for_alias(token.text())?;
                Some(SemanticEq::Register(arch::registers::register_id(
                    register,
                    &self.config,
                )?))
            }
            SyntaxKind::TOKEN => Some(SemanticEq::String(token.text())),
            SyntaxKind::NUMBER => Some(SemanticEq::Numeric(token.text().parse::<i128>().ok()?)),
            SyntaxKind::FLOAT => Some(SemanticEq::Float(token.text().parse::<f64>().ok()?)),
            SyntaxKind::LABEL => self
                .token::<LabelToken>(token)
                .map(|t| SemanticEq::String(t.name())),
            SyntaxKind::RELOCATION => Some(SemanticEq::String(token.text())),
            SyntaxKind::CONSTANT => Some(SemanticEq::String(token.text())),
            SyntaxKind::NAME => Some(SemanticEq::String(token.text())),
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
            | SyntaxKind::IMMEDIATE
            | SyntaxKind::ALIAS
            | SyntaxKind::INSTRUCTION
            | SyntaxKind::DIRECTIVE
            | SyntaxKind::BRACKETS
            | SyntaxKind::METADATA
            | SyntaxKind::OBJDUMP_OFFSET
            | SyntaxKind::EXPR
            | SyntaxKind::CONST_DEF
            | SyntaxKind::ROOT => None,
        }
    }

    /// Attempt to determine the architecture that the assembly data is for.
    fn determine_architecture(filedata: &str, config: &LSPConfig) -> Architecture {
        use regex::Regex;

        static ARCH_DETECTION: Lazy<[Regex; 4]> = Lazy::new(|| {
            [
                Regex::new(r#"lsp-asm-architecture: (.*) ?"#).unwrap(),
                Regex::new(r#"^\s*\.arch (.*)"#).unwrap(),
                Regex::new(r#".*:[\t ]+file format elf64-(.*)"#).unwrap(),
                Regex::new(r#".*:[\t ]+file format (?i)mach-o (.*)"#).unwrap(),
            ]
        });

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

    pub(crate) fn config_from_arch(arch: &Architecture) -> ParserConfig {
        match arch {
            Architecture::AArch64 => ParserConfig {
                comment_start: String::from("//"),
                registers: Some(&arch::register_names::AARCH64_REGISTERS),
                architecture: *arch,
                ..ParserConfig::default()
            },
            Architecture::X86_64 => ParserConfig {
                registers: Some(&arch::register_names::X86_64_REGISTERS),
                architecture: *arch,
                ..ParserConfig::default()
            },
            Architecture::Unknown => ParserConfig::default(),
        }
    }

    #[cfg(test)]
    pub fn in_memory(data: &str, config: &LSPConfig) -> Self {
        let (parser, _) = Self::from(Url::parse("memory://ignore").unwrap(), data, config);
        parser
    }
}

pub fn split_parsed_include(include: ParsedInclude) -> (Parser, Vec<ParsedInclude>) {
    let filesize = Byte::from_bytes(include.data.len() as _);

    (
        Parser {
            id: Url::parse(&include.id).unwrap(),
            line_index: PositionInfo::new(&include.data),
            filesize,
            root: include.root,
            config: Default::default(),
            debug_map: OnceCell::new(),
            alias: include.alias,
        },
        include.included_files,
    )
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

    pub fn offset_for_line(&self, line: LineNumber) -> Option<u32> {
        self.lines
            .get(line as usize)
            .map(|s: &TextSize| u32::from(*s))
    }

    pub fn range_to_text_range(&self, range: &DocumentRange) -> Option<TextRange> {
        let start = self.point_for_position(&range.start)?;
        let end = self.point_for_position(&range.end)?;

        Some(TextRange::new(start, end))
    }

    /// Helper function to get the line and column for a text size
    pub fn get_position_for_size(&self, ts: &TextSize) -> Option<DocumentPosition> {
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
            .chain(std::iter::once(((data.len() + 1) as u32).into()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_map() {
        let data = r#"first line
second line
another"#;
        let map = PositionInfo::new(data);
        let result: Vec<TextSize> = vec![0u32.into(), 11u32.into(), 23u32.into(), 31u32.into()];
        assert_eq!(result, map.lines);
    }
}
