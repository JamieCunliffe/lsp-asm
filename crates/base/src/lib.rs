#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod architecture;
pub use architecture::*;

mod filetype;
pub use filetype::*;

use serde::Deserialize;
use serde::Deserializer;

pub mod register;

pub fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}
