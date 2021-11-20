use std::collections::HashMap;
use std::fmt::Debug;

use base::{null_as_default, Architecture};
use byte_unit::{Byte, ByteUnit};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LSPConfig {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub architecture: Architecture,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub codelens: CodelensConfig,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub analysis: AnalysisConfig,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub diagnostics: DiagnosticsConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodelensConfig {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub enabled_filesize: Byte,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub loc_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisConfig {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub default_cpus: HashMap<Architecture, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsConfig {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub enabled: bool,
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

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
