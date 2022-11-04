#![allow(clippy::upper_case_acronyms)]

use base::register::{RegisterKind, RegisterSize, Registers};
use unicase::UniCase;

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
            "a" => RegisterSize::Any,
            _ => RegisterSize::Unknown,
        }
    }

    fn is_sp(&self, name: &str) -> bool {
        name.contains("sp")
    }
}

pub const DOC_REGISTERS: DocRegisters = DocRegisters {};

pub fn to_documentation_name(kind: &RegisterKind, size: &RegisterSize) -> String {
    format!("<{kind}_{size}>")
}

pub static DOCUMENTATION_REGISTERS: phf::Map<UniCase<&'static str>, i8> = phf::phf_map! {
    UniCase::ascii("<fp_128>") => 0,
    UniCase::ascii("<fp_16>") => 0,
    UniCase::ascii("<fp_32>") => 0,
    UniCase::ascii("<fp_64>") => 0,
    UniCase::ascii("<fp_8>") => 0,
    UniCase::ascii("<gp_32>") => 0,
    UniCase::ascii("<gp_64>") => 0,
    UniCase::ascii("<gp_a>") => 0,
    UniCase::ascii("<gp|sp_64>") => 0,
    UniCase::ascii("<gp|sp_a>") => 0,
    UniCase::ascii("<pred_v>") => 0,
    UniCase::ascii("<scale_v>") => 0,
    UniCase::ascii("<simd_v>") => 0,
};

#[cfg(test)]
mod tests {
    use super::{DOCUMENTATION_REGISTERS, DOC_REGISTERS};
    use base::register::{RegisterSize, Registers};
    use pretty_assertions::assert_ne;

    #[test]
    fn test_documentation_registers_are_known() {
        for register in DOCUMENTATION_REGISTERS.keys() {
            assert!(!DOC_REGISTERS.get_kind(register).is_empty());
            assert_ne!(RegisterSize::Unknown, DOC_REGISTERS.get_size(register));
        }
    }
}
