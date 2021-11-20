use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use lsp_types::Url;
use uuid::Uuid;

pub struct UrlPath(PathBuf);
impl TryFrom<&Url> for UrlPath {
    type Error = std::io::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        Ok(UrlPath(std::fs::canonicalize(
            value.to_file_path().map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Failed to convert uri to file path",
                )
            })?,
        )?))
    }
}

impl UrlPath {
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    pub fn is_file(&self, file: &str) -> bool {
        let file = Path::new(file);
        std::fs::canonicalize(file)
            .ok()
            .map(|full_file| full_file == self.0)
            .unwrap_or(false)
    }
}

pub(super) fn run_command(binary: &str, args: &[String]) -> Option<String> {
    let result = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?
        .wait_with_output()
        .ok()?;

    String::from_utf8(result.stderr).ok()
}

pub(super) struct TemporaryFile {
    file: String,
}
impl TemporaryFile {
    pub fn new() -> Self {
        let file = if cfg!(target_family = "windows") {
            let mut dir = std::env::temp_dir();
            dir.push(Uuid::new_v4().to_string());
            dir.to_str().unwrap_or_default().to_string()
        } else {
            String::from("/dev/null")
        };

        Self { file }
    }

    pub fn filename(&self) -> &String {
        &self.file
    }
}
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.filename() != "/dev/null" {
            let _ = std::fs::remove_file(self.filename());
        }
    }
}
