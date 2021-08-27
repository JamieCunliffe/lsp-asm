#![allow(clippy::upper_case_acronyms)]

use base::register::{RegisterKind, RegisterSize, Registers};
use parser::Register;

pub struct DocRegisters {}

impl Registers for DocRegisters {
    fn get_kind(&self, name: &str) -> RegisterKind {
        let kind = name.split('_').next().unwrap_or("\0");

        if kind.get(0..=0) == Some("<") {
            match kind.get(1..).unwrap_or("\0") {
                "GP" => RegisterKind::GENERAL_PURPOSE,
                "GP|SP" => RegisterKind::GP_OR_SP,
                "FP" => RegisterKind::FLOATING_POINT,
                "SIMD" => RegisterKind::SIMD,
                "SCALE" => RegisterKind::SCALABLE,
                "PRED" => RegisterKind::PREDICATE,
                _ => RegisterKind::NONE,
            }
        } else {
            RegisterKind::NONE
        }
    }

    fn get_size(&self, name: &str) -> RegisterSize {
        let size = name.split('_').nth(1).unwrap_or("\0").trim_end_matches('>');

        match size {
            "8" => RegisterSize::Bits8,
            "16" => RegisterSize::Bits16,
            "32" => RegisterSize::Bits32,
            "64" => RegisterSize::Bits64,
            "128" => RegisterSize::Bits128,
            "V" => RegisterSize::Vector,
            "U" => RegisterSize::Unknown,
            _ => RegisterSize::Unknown,
        }
    }

    fn is_sp(&self, name: &str) -> bool {
        name.contains("SP")
    }
}

pub const DOC_REGISTERS: DocRegisters = DocRegisters {};

pub fn to_documentation_name(kind: &RegisterKind, size: &RegisterSize) -> String {
    format!("<{kind}_{size}>", kind = kind, size = size)
}

pub const DOCUMENTATION_REGISTERS: [Register; 1] = [Register::new(&[
    "<FP_128>",
    "<FP_16>",
    "<FP_32>",
    "<FP_64>",
    "<FP_8>",
    "<GP_32>",
    "<GP_64>",
    "<GP_U>",
    "<GP|SP_64>",
    "<PRED_V>",
    "<SCALE_V>",
    "<SIMD_V>",
])];
