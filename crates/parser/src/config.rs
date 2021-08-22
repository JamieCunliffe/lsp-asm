use crate::Register;
use base::{Architecture, FileType};

/// Configuration for the parser
#[derive(Clone, PartialEq, Debug)]
pub struct ParserConfig {
    /// The characters that signal the start of a comment
    pub comment_start: String,

    pub architecture: Architecture,

    /// The filetype of this parser
    pub file_type: FileType,

    /// The registers that are allowed for this parser
    pub registers: Option<&'static [Register]>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            comment_start: "#".to_string(),
            architecture: Architecture::Unknown,
            file_type: Default::default(),
            registers: None,
        }
    }
}
