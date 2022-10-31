pub mod access;
mod map;
pub mod registers;
pub mod templates;

use base::{null_as_default, Architecture};
use itertools::{Either, Itertools};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::ops::Range;
use std::sync::{Arc, RwLock};

pub use map::*;

pub type CompletionValue = Either<String, Range<i64>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OperandInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub completion_values: Option<Vec<CompletionValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OperandAccessType {
    Write,
    Read,
    Text,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InstructionTemplate {
    pub asm: Vec<String>,
    pub display_asm: String,
    pub items: Vec<OperandInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub access_map: Vec<OperandAccessType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction {
    pub opcode: String,
    pub header: Option<String>,
    pub architecture: Option<String>,
    pub description: String,
    pub asm_template: Vec<InstructionTemplate>,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(header) = self.header.clone() {
            writeln!(f, "# {header}\n")?;
        }

        writeln!(f, "{}", self.description)?;

        if !self.asm_template.is_empty() {
            writeln!(
                f,
                r#"
## Syntax:

{}"#,
                self.asm_template
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect_vec()
                    .join("\n")
            )?;
        }

        Ok(())
    }
}

impl Display for InstructionTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "* `{}`\n{}",
            self.display_asm,
            self.items
                .iter()
                .map(|item| format!("{item}"))
                .collect_vec()
                .join("\n")
        )
    }
}

impl Display for OperandInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  - **{}** {}", self.name, self.description)
    }
}

#[derive(Debug)]
pub struct CacheError {
    pub reason: String,
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

static DOCUMENTATION_CACHE: Lazy<RwLock<HashMap<Architecture, Arc<DocumentationMap>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn load_documentation(arch: &Architecture) -> Result<Arc<DocumentationMap>, Box<dyn Error>> {
    {
        let cache = DOCUMENTATION_CACHE.read()?;
        if let Some(d) = cache.get(arch) {
            return Ok(d.clone());
        }
    }
    let base = directories::BaseDirs::new().ok_or_else(|| CacheError {
        reason: String::from("Failed to init base directories"),
    })?;
    let path = base
        .data_local_dir()
        .join("lsp-asm")
        .join(format!("{arch}.json"));

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader).map_err(|e| {
        log::error!(
            "Failed to parse documentation due to error: {}",
            e.to_string()
        );
        e
    })?;

    {
        let mut cache = DOCUMENTATION_CACHE.write()?;
        cache.insert(*arch, Arc::new(data));
    }

    if let Some(d) = DOCUMENTATION_CACHE.read()?.get(arch) {
        Ok(d.clone())
    } else {
        Err(Box::new(CacheError {
            reason: String::from("Failed to read back inserted documentation"),
        }))
    }
}

#[cfg(feature = "poison")]
pub fn poison_cache(arch: &Architecture, data: DocumentationMap) {
    let mut cache = DOCUMENTATION_CACHE.write().unwrap();
    cache.insert(*arch, Arc::new(data));
}

#[cfg(test)]
mod tests;
