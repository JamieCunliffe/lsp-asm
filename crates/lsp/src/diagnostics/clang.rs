use itertools::Itertools;

use crate::types::{ColumnNumber, LineNumber};

use super::util::{run_command, TemporaryFile};
use super::{Assembler, CompileCommand, Error};

pub struct Clang {
    pub command: CompileCommand,
}

impl Clang {
    fn process_errors(errors: String) -> Vec<Error> {
        errors
            .lines()
            .filter_map(Self::process_error_line)
            .collect_vec()
    }

    fn process_error_line(error: &str) -> Option<Error> {
        let file = error
            .split(':')
            .enumerate()
            .take_while(|(idx, a)| *idx == 0 || (a.starts_with('/') || a.starts_with('\\')))
            .map(|(_, a)| a)
            .join(":");

        if file.len() == error.len() {
            return None;
        }

        let mut colons = error[file.len() + 1..].split(':');

        let line = colons.next()?.parse::<LineNumber>().map(|l| l - 1).ok()?;
        let column = colons.next()?.parse::<ColumnNumber>().ok()?;
        let level = colons.next()?.trim().into();
        let description = colons.join(":").trim().to_string();

        Some(Error {
            code: "".into(),
            description,
            file,
            line,
            column,
            level,
        })
    }
}

impl Assembler for Clang {
    fn get_errors(&self) -> Vec<Error> {
        let mut args = self.command.get_arguments().to_vec();
        let command = self.command.get_command();

        let temp_file = TemporaryFile::new();
        args.push(String::from("-o"));
        args.push(temp_file.filename().clone());

        run_command(command, &args)
            .map(Self::process_errors)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::clang::Clang;
    use crate::diagnostics::{Error, ErrorLevel};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_clang_error_line() {
        let expected = Error {
            file: String::from("test.s"),
            line: 39,
            column: 11,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("mach-o section specifier uses an unknown section type"),
        };

        assert_eq!(
            Clang::process_error_line(
                "test.s:40:11: error: mach-o section specifier uses an unknown section type"
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn test_clang_error_colon() {
        let expected = Error {
            file: String::from("test.s"),
            line: 80,
            column: 2,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("instruction requires: FEATURE"),
        };

        assert_eq!(
            Clang::process_error_line("test.s:81:2: error: instruction requires: FEATURE").unwrap(),
            expected
        );
    }

    #[test]
    fn test_clang_error_win_path() {
        let expected = Error {
            file: String::from("C:\\test.s"),
            line: 0,
            column: 5,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("unknown directive"),
        };

        assert_eq!(
            Clang::process_error_line("C:\\test.s:1:5: error: unknown directive").unwrap(),
            expected
        );
    }

    #[test]
    fn test_clang_error() {
        let error = r#"test.s:3:9: error: unknown directive
        .this_is_bad
        ^
"#;
        let expected = Error {
            file: String::from("test.s"),
            line: 2,
            column: 9,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("unknown directive"),
        };

        assert_eq!(Clang::process_errors(String::from(error)), vec![expected]);
    }
}
