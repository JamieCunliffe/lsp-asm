use std::str::FromStr;

use lsp_types::{TextDocumentIdentifier, Url};

use crate::util;

#[derive(Clone, Debug)]
pub struct FileUrl {
    uri: Url,
}

impl FileUrl {
    pub fn to_text_document(&self) -> TextDocumentIdentifier {
        TextDocumentIdentifier {
            uri: self.uri.clone(),
        }
    }
}

impl FromStr for FileUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            uri: util::file_to_uri(s),
        })
    }
}

impl From<FileUrl> for Url {
    fn from(val: FileUrl) -> Self {
        val.uri
    }
}

impl AsRef<Url> for FileUrl {
    fn as_ref(&self) -> &Url {
        &self.uri
    }
}
