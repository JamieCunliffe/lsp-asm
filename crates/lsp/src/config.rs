use std::fmt::Debug;

use serde::{Deserialize, Deserializer};

use crate::types::Architecture;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LSPConfig {
    pub architecture: Architecture,
}

impl<'de> Deserialize<'de> for Architecture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arch: String = Deserialize::deserialize(deserializer)?;

        Ok(match Architecture::from(arch.as_str()) {
            Architecture::Unknown => Architecture::default(),
            arch => arch,
        })
    }
}
