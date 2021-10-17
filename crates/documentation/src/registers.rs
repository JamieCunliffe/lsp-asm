#![allow(clippy::upper_case_acronyms)]

use base::register::{RegisterKind, RegisterSize, Registers};
use parser::Register;

pub struct DocRegisters {}

impl Registers for DocRegisters {
    fn get_kind(&self, name: &str) -> RegisterKind {
        let kind = name.split('_').next().unwrap_or("\0");

        if kind.get(0..=0) == Some("<") {
            match kind.get(1..).unwrap_or("\0") {
                "gp" => RegisterKind::GENERAL_PURPOSE,
                "gp|sp" => RegisterKind::GP_OR_SP,
                "fp" => RegisterKind::FLOATING_POINT,
                "simd" => RegisterKind::SIMD,
                "scale" => RegisterKind::SCALABLE,
                "pred" => RegisterKind::PREDICATE,
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
            "v" => RegisterSize::Vector,
            "u" => RegisterSize::Unknown,
            _ => RegisterSize::Unknown,
        }
    }

    fn is_sp(&self, name: &str) -> bool {
        name.contains("sp")
    }
}

pub const DOC_REGISTERS: DocRegisters = DocRegisters {};

pub fn to_documentation_name(kind: &RegisterKind, size: &RegisterSize) -> String {
    format!("<{kind}_{size}>", kind = kind, size = size)
}

pub const DOCUMENTATION_REGISTERS: [Register; 1] = [Register::new(&[
    "<fp_128>",
    "<fp_16>",
    "<fp_32>",
    "<fp_64>",
    "<fp_8>",
    "<gp_32>",
    "<gp_64>",
    "<gp_u>",
    "<gp|sp_64>",
    "<pred_v>",
    "<scale_v>",
    "<simd_v>",
])];
