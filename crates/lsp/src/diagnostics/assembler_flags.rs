use std::convert::TryInto;
use std::path::PathBuf;
use std::str::FromStr;

use itertools::Itertools;

use super::{CompileCommand, Diagnostics};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct AssemblerFlags {
    root: String,
    command: String,
    arguments: Vec<String>,
}

impl AssemblerFlags {
    pub fn new(root: &str) -> Option<Self> {
        let mut path = PathBuf::from_str(root).ok()?;
        path.push("assembler_flags.txt");

        let data = std::fs::read_to_string(path).ok()?;
        Self::from_data(root.to_string(), &data)
    }

    fn from_data(root: String, data: &str) -> Option<Self> {
        let data = data.lines().map(String::from).collect_vec();
        let (cmd, args) = data.split_first()?;

        Some(Self {
            root,
            command: cmd.clone(),
            arguments: args.to_vec(),
        })
    }
}

impl Diagnostics for AssemblerFlags {
    fn assembler_for_file(&self, uri: &lsp_types::Url) -> Option<Box<dyn super::Assembler>> {
        let mut arguments = self.arguments.clone();
        arguments.push(uri.to_file_path().ok()?.as_os_str().to_str()?.to_string());

        let command = CompileCommand {
            file: Default::default(),
            directory: Default::default(),
            command: self.command.clone(),
            arguments,
        };

        command.try_into().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::AssemblerFlags;

    #[test]
    fn parse_flags() {
        let data = r#"binary
arg1
-arg
another"#;

        let flags = AssemblerFlags::from_data(String::from("unit-test"), data).unwrap();
        assert_eq!(
            flags,
            AssemblerFlags {
                root: String::from("unit-test"),
                command: String::from("binary"),
                arguments: vec![
                    String::from("arg1"),
                    String::from("-arg"),
                    String::from("another"),
                ],
            },
        );
    }

    #[test]
    fn parse_flags_no_args() {
        let data = r#"binary"#;

        let flags = AssemblerFlags::from_data(String::from("unit-test"), data).unwrap();
        assert_eq!(
            flags,
            AssemblerFlags {
                root: String::from("unit-test"),
                command: String::from("binary"),
                arguments: vec![],
            },
        );
    }
}
