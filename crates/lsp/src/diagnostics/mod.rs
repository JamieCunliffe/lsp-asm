use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use std::convert::TryInto;

use crate::types::{ColumnNumber, LineNumber};

pub use self::util::UrlPath;

pub mod assembler_flags;
mod clang;
pub mod compile_commands;
mod gcc;
mod util;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
    Info,
}

impl From<&str> for ErrorLevel {
    fn from(err: &str) -> Self {
        match err.to_lowercase().as_str() {
            "error" => Self::Error,
            "warn" => Self::Warning,
            _ => Self::Info,
        }
    }
}

impl From<ErrorLevel> for DiagnosticSeverity {
    fn from(val: ErrorLevel) -> Self {
        match val {
            ErrorLevel::Error => DiagnosticSeverity::ERROR,
            ErrorLevel::Warning => DiagnosticSeverity::WARNING,
            ErrorLevel::Info => DiagnosticSeverity::INFORMATION,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub file: String,
    pub line: LineNumber,
    pub column: ColumnNumber,
    pub level: ErrorLevel,
    pub code: String,
    pub description: String,
}

impl From<Error> for Diagnostic {
    fn from(val: Error) -> Self {
        Diagnostic {
            range: Range {
                start: Position::new(val.line, val.column),
                end: Position::new(val.line, val.column),
            },
            severity: Some(val.level.into()),
            code: None,
            code_description: None,
            source: None,
            message: val.description,
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

pub trait Assembler {
    fn get_errors(&self) -> Vec<Error>;
}

pub trait Diagnostics {
    fn assembler_for_file(&self, uri: &Url) -> Option<Box<dyn Assembler>>;
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct CompileCommand {
    file: String,
    directory: String,
    command: String,
    arguments: Vec<String>,
}

impl CompileCommand {
    pub(crate) fn get_arguments(&self) -> &[String] {
        self.arguments.as_slice()
    }

    pub(crate) fn get_command(&self) -> &String {
        &self.command
    }

    fn is_uri(&self, uri: Option<UrlPath>) -> bool {
        uri.map(|uri| uri.is_file(&self.file)).unwrap_or(false)
    }
}

impl TryInto<Box<dyn Assembler>> for CompileCommand {
    type Error = String;

    fn try_into(self) -> Result<Box<dyn Assembler>, Self::Error> {
        let cmd = &self.command;
        if cmd.contains("clang") {
            Ok(Box::new(crate::diagnostics::clang::Clang { command: self }))
        } else if cmd.contains("gcc") || cmd.contains("g++") {
            Ok(Box::new(crate::diagnostics::gcc::Gcc { command: self }))
        } else {
            warn!("`{cmd}` is not a known assembler");
            Err(format!("`{cmd}` is not a known assembler"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_uri() {
        let uri = Url::from_file_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/diagnostics/mod.rs"
        ))
        .unwrap();

        let cc = CompileCommand {
            file: String::from("src/diagnostics/mod.rs"),
            directory: String::from("dir"),
            command: String::from("app"),
            arguments: vec![String::from("arg1"), String::from("arg2")],
        };
        assert!(cc.is_uri((&uri).try_into().ok()));

        let cc = CompileCommand {
            file: String::from("src/diagnostics/not_a_real_file.rs"),
            ..cc
        };
        assert!(!cc.is_uri((&uri).try_into().ok()));
    }
}
