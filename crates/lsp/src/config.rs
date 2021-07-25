use std::fmt::Debug;

use byte_unit::{Byte, ByteUnit};
use serde::{Deserialize, Deserializer};

use crate::types::Architecture;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LSPConfig {
    pub architecture: Architecture,
    pub codelens: CodelensConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CodelensConfig {
    pub enabled_filesize: Byte,
    pub loc_enabled: bool,
}

impl Default for CodelensConfig {
    fn default() -> Self {
        Self {
            enabled_filesize: Byte::from_unit(1., ByteUnit::MiB).unwrap(),
            loc_enabled: true,
        }
    }
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
