use base::Architecture;
use documentation::Instruction;
use itertools::Itertools;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;

mod aarch64;
mod register_replacements;
mod util;

pub async fn build_aarch64_instructions() -> Result<(), Box<dyn Error + Sync + Send>> {
    println!("Building AArch64 instruction set reference");

    let mut entries = match aarch64::get_instructions().await {
        Ok(e) => e,
        Err(e) => panic!("Process aarch64 failed: {e}"),
    };
    write_entries(&Architecture::AArch64, &mut entries)?;

    Ok(())
}

fn make_hash_map(instructions: &mut Vec<Instruction>) -> HashMap<String, Vec<Instruction>> {
    instructions
        .drain(..)
        .into_group_map_by(|i| i.opcode.to_lowercase())
}

fn write_entries(
    arch: &Architecture,
    instructions: &mut Vec<Instruction>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let base = directories::BaseDirs::new().unwrap();
    let mut path = base.data_local_dir().join("lsp-asm");

    info!("Attempting to create directory: {:#?}", path);
    let _ = fs::create_dir_all(&path);

    info!("Converting vector into hashmap");
    let instructions = make_hash_map(instructions);

    path.push(format!("{arch}.json"));
    info!("Attempting to write json file to: {:#?}", path);
    let mut file = File::create(path)?;

    let serialized = serde_json::to_string(&instructions)?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}
