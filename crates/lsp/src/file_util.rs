use std::path::{Path, PathBuf};

use lsp_types::Url;

pub fn make_file_relative(base: &str, file: &str) -> Option<PathBuf> {
    let tmp = Path::new(file);
    if tmp.is_absolute() {
        return Some(tmp.to_path_buf());
    }

    let from = Url::parse(base)
        .ok()
        .and_then(|uri| uri.to_file_path().ok())
        .unwrap_or_else(|| Path::new(base.trim_start_matches("file://")).to_path_buf());

    let from = Path::new(&from);
    let from_dir = from.parent()?;
    let mut from_dir = from_dir.to_path_buf();
    from_dir.push(&file);
    Some(from_dir)
}
