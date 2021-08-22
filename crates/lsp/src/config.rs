use std::collections::HashMap;
use std::fmt::Debug;

use byte_unit::{Byte, ByteUnit};
use serde::{Deserialize, Deserializer};

use base::Architecture;

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

fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}
