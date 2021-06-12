use crate::types::Architecture;

use super::registers::Register;

/// Configuration for the parser
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ParserConfig {
    /// The characters that signal the start of a comment
    pub comment_start: String,

    /// The filetype of this parser
    pub file_type: FileType,

    /// The registers that are allowed for this parser
    pub registers: Option<&'static [Register]>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            comment_start: "#".to_string(),
            file_type: Default::default(),
            registers: None,
        }
    }
}

impl ParserConfig {
    pub fn new(arch: &Architecture) -> Self {
        match arch {
            Architecture::AArch64 => Self {
                comment_start: String::from("//"),
                registers: Some(&super::registers::AARCH64_REGISTERS),
                ..Self::default()
            },
            Architecture::X86_64 => Self {
                registers: Some(&super::registers::X86_64_REGISTERS),
                ..Self::default()
            },
            Architecture::Unknown => Self::default(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum FileType {
    Assembly,
    ObjDump,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Assembly
    }
}
