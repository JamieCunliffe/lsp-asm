use std::collections::HashMap;
use std::fmt::Debug;

use byte_unit::{Byte, ByteUnit};
use serde::{Deserialize, Deserializer};

use crate::types::Architecture;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LSPConfig {
    pub architecture: Architecture,
    pub codelens: CodelensConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CodelensConfig {
    pub enabled_filesize: Byte,
    pub loc_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AnalysisConfig {
    #[serde(deserialize_with = "null_as_default")]
    pub default_cpus: HashMap<Architecture, String>,
}

impl Default for CodelensConfig {
    fn default() -> Self {
        Self {
            enabled_filesize: Byte::from_unit(1., ByteUnit::MiB).unwrap(),
            loc_enabled: true,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            default_cpus: Default::default(),
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

fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}
