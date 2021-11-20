#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod register;
pub use architecture::*;
pub use filetype::*;

mod architecture;
mod filetype;

use serde::{Deserialize, Deserializer};

pub fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}
