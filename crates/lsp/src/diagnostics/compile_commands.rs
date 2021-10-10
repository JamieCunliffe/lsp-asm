use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use super::{Assembler, CompileCommand, Diagnostics};
use itertools::Itertools;
use lsp_types::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct SerializeCompileCommand {
    pub directory: String,
    pub command: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub file: String,
}

impl TryInto<CompileCommand> for SerializeCompileCommand {
    type Error = ();

    fn try_into(self) -> Result<CompileCommand, Self::Error> {
        let args = self.arguments;
        let cmd = self.command;
        let command = args.or_else(move || cmd.map(|cmd| shellwords::split(&cmd).ok()).flatten());

        if let Some(command) = command {
            if let Some((cmd, args)) = command.split_first() {
                Ok(CompileCommand {
                    file: self.file,
                    directory: self.directory,
                    command: cmd.clone(),
                    arguments: args.to_vec(),
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[derive(Default, Debug)]
pub struct CompileCommands {
    pub commands: Vec<CompileCommand>,
}

impl CompileCommands {
    pub fn new(root: &str) -> Option<Self> {
        let mut path = PathBuf::from_str(root).ok()?;
        path.push("compile_commands.json");

        let data = fs::read_to_string(path).ok()?;
        Self::from_data(&data)
    }

    fn from_data(data: &str) -> Option<Self> {
        let commands = serde_json::from_str::<Vec<SerializeCompileCommand>>(data)
            .ok()?
            .into_iter()
            .filter_map(|c| c.try_into().ok())
            .collect_vec();

        Some(Self { commands })
    }
}

impl Diagnostics for CompileCommands {
    fn assembler_for_file(&self, uri: &Url) -> Option<Box<dyn Assembler>> {
        let command = self
            .commands
            .iter()
            .find(|c| c.is_uri(uri.try_into().ok()))?;

        command.clone().try_into().ok()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_command() {
        let cc = SerializeCompileCommand {
            directory: String::from("dir"),
            command: Some(String::from("app arg1 arg2")),
            arguments: Default::default(),
            file: String::from("file"),
        };
        let expected = CompileCommand {
            file: String::from("file"),
            directory: String::from("dir"),
            command: String::from("app"),
            arguments: vec![String::from("arg1"), String::from("arg2")],
        };
        let actual: CompileCommand = cc.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_arguments() {
        let cc = SerializeCompileCommand {
            directory: String::from("/"),
            command: Default::default(),
            arguments: Some(vec![
                String::from("application"),
                String::from("a1"),
                String::from("-o"),
            ]),
            file: String::from("f1"),
        };
        let expected = CompileCommand {
            file: String::from("f1"),
            directory: String::from("/"),
            command: String::from("application"),
            arguments: vec![String::from("a1"), String::from("-o")],
        };
        let actual: CompileCommand = cc.try_into().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_compile_commands() {
        let json = r#"[
    {
        "arguments": [
            "app",
            "a1",
            "a2"
        ],
        "directory": "/",
        "file": "file1"
    },
    {
        "command": "/usr/bin/c++  arg1 arg2",
        "directory": "/usr/",
        "file": "file2"
    }
]"#;
        let commands = CompileCommands::from_data(json);
        let expected = vec![
            CompileCommand {
                file: String::from("file1"),
                arguments: vec![String::from("a1"), String::from("a2")],
                command: String::from("app"),
                directory: String::from("/"),
            },
            CompileCommand {
                file: String::from("file2"),
                directory: String::from("/usr/"),
                command: String::from("/usr/bin/c++"),
                arguments: vec![String::from("arg1"), String::from("arg2")],
            },
        ];

        assert_eq!(commands.unwrap().commands, expected);
    }
}
