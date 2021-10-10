use itertools::Itertools;

use crate::types::LineNumber;

use super::util::{run_command, TemporaryFile};
use super::{Assembler, CompileCommand, Error};

pub struct Gcc {
    pub command: CompileCommand,
}

impl Gcc {
    fn process_errors(errors: String) -> Vec<Error> {
        errors
            .lines()
            .skip(1)
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
        let column = 0;
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

impl Assembler for Gcc {
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
    use crate::diagnostics::gcc::Gcc;
    use crate::diagnostics::{Error, ErrorLevel};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_gcc_single_error() {
        let expected = Error {
            file: String::from("file.s"),
            line: 12,
            column: 0,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("bad register name `%rbp1'"),
        };

        assert_eq!(
            Gcc::process_error_line("file.s:13: Error: bad register name `%rbp1'").unwrap(),
            expected
        );
    }

    #[test]
    fn test_gcc_error_win_path() {
        let expected = Error {
            file: String::from("C:\\test.s"),
            line: 0,
            column: 0,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("unknown pseudo-op: `.test'"),
        };

        assert_eq!(
            Gcc::process_error_line("C:\\test.s:1: Error: unknown pseudo-op: `.test'").unwrap(),
            expected
        );
    }

    #[test]
    fn test_gcc_errors() {
        let errors = r#"file.s: Assembler messages:
file.s:16: Error: bad register name `%rbp1'"#;

        let expected = Error {
            file: String::from("file.s"),
            line: 15,
            column: 0,
            level: ErrorLevel::Error,
            code: Default::default(),
            description: String::from("bad register name `%rbp1'"),
        };

        assert_eq!(Gcc::process_errors(String::from(errors)), vec![expected]);
    }
}
