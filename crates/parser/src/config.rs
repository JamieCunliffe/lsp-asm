use base::{Architecture, FileType};
use unicase::UniCase;

/// Configuration for the parser
#[derive(Clone, Debug)]
pub struct ParserConfig {
    /// The characters that signal the start of a comment
    pub comment_start: String,

    pub architecture: Architecture,

    /// The filetype of this parser
    pub file_type: FileType,

    /// The registers that are allowed for this parser
    pub registers: Option<&'static phf::Map<UniCase<&'static str>, i8>>,
}

impl PartialEq for ParserConfig {
    fn eq(&self, other: &Self) -> bool {
        self.comment_start == other.comment_start
            && self.architecture == other.architecture
            && self.file_type == other.file_type
    }
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
