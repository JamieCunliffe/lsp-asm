use base::register::{RegisterKind, RegisterSize, Registers};
use base::Architecture;
use itertools::Itertools;
use parser::config::ParserConfig;
use unicase::UniCase;

pub struct AArch64 {}
impl Registers for AArch64 {
    fn get_kind(&self, name: &str) -> RegisterKind {
        let name = name.to_lowercase();

        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterKind::GENERAL_PURPOSE,
            "w" => RegisterKind::GENERAL_PURPOSE,
            "s" if self.is_sp(&name) => RegisterKind::SP,
            "q" => RegisterKind::FLOATING_POINT,
            "d" => RegisterKind::FLOATING_POINT,
            "s" if !self.is_sp(&name) => RegisterKind::FLOATING_POINT,
            "h" => RegisterKind::FLOATING_POINT,
            "b" => RegisterKind::FLOATING_POINT,
            "v" => {
                let size = name.split('.').next().unwrap_or("\0");
                match size {
                    "8b" | "16b" => RegisterKind::SIMD,
                    "4h" | "8h" => RegisterKind::SIMD,
                    "2s" | "4s" => RegisterKind::SIMD,
                    "2d" => RegisterKind::SIMD,
                    _ => RegisterKind::SIMD,
                }
            }
            "p" => RegisterKind::PREDICATE,
            "z" => RegisterKind::SCALABLE,
            _ => RegisterKind::NONE,
        }
    }

    fn get_size(&self, name: &str) -> RegisterSize {
        let name = name.to_lowercase();

        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterSize::Bits64,
            "s" if self.is_sp(&name) => RegisterSize::Bits64,
            "w" => RegisterSize::Bits32,
            "q" => RegisterSize::Bits128,
            "d" => RegisterSize::Bits64,
            "s" => RegisterSize::Bits32,
            "h" => RegisterSize::Bits16,
            "b" => RegisterSize::Bits8,
            "v" => {
                let size = name.split('.').next().unwrap_or("\0");
                match size {
                    "8b" | "16b" => RegisterSize::Vector,
                    "4h" | "8h" => RegisterSize::Vector,
                    "2s" | "4s" => RegisterSize::Vector,
                    "2d" => RegisterSize::Vector,
                    _ => RegisterSize::Vector,
                }
            }
            "p" => RegisterSize::Vector,
            "z" => RegisterSize::Vector,
            _ => RegisterSize::Unknown,
        }
    }

    fn is_sp(&self, name: &str) -> bool {
        name.eq_ignore_ascii_case("sp")
    }
}

pub struct UnknownRegisters {}
impl Registers for UnknownRegisters {
    fn get_kind(&self, _register: &str) -> RegisterKind {
        RegisterKind::NONE
    }

    fn get_size(&self, _register: &str) -> RegisterSize {
        RegisterSize::Unknown
    }

    fn is_sp(&self, _register: &str) -> bool {
        false
    }
}

pub fn registers_for_architecture(arch: &Architecture) -> &dyn Registers {
    static REGISTER_AARCH64: &AArch64 = &AArch64 {};
    static REGISTER_NONE: &UnknownRegisters = &UnknownRegisters {};
    match arch {
        Architecture::AArch64 => REGISTER_AARCH64,
        Architecture::X86_64 => REGISTER_NONE,
        Architecture::Unknown => REGISTER_NONE,
    }
}

#[derive(Debug)]
pub struct RegisterList {
    map: &'static phf::Map<UniCase<&'static str>, i8>,
}

impl RegisterList {
    pub fn from_architecture(arch: &Architecture) -> Self {
        Self {
            map: match arch {
                Architecture::AArch64 => &crate::register_names::AARCH64_REGISTERS,
                Architecture::X86_64 => &crate::register_names::X86_64_REGISTERS,
                Architecture::Unknown => &crate::register_names::UNKNOWN_REGISTERS,
            },
        }
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        self.map.keys().map(|x| x.as_ref()).sorted()
    }
}

/// Gets an index for this register that can be used for comparisons, the id
/// that is returned should only be considered valid for the given parser config
/// when comparing.
pub fn register_id(name: &str, config: &ParserConfig) -> Option<i8> {
    if let Some(registers) = config.registers {
        let name = parser::register_name(name);
        registers.get(&UniCase::ascii(name)).cloned()
    } else {
        None
    }
}
