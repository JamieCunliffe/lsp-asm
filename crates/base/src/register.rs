use std::fmt::Display;

use bitflags::bitflags;

bitflags! {
    pub struct RegisterKind : u32 {
        const NONE            = 0b00000000;
        const GENERAL_PURPOSE = 0b00000001;
        const FLOATING_POINT  = 0b00000010;
        const SIMD            = 0b00000100;
        const SCALABLE        = 0b00001000;
        const PREDICATE       = 0b00010000;
        const SP              = 0b00100000;

        const GP_OR_SP = RegisterKind::GENERAL_PURPOSE.bits | RegisterKind::SP.bits;
    }
}

#[derive(Debug, PartialEq)]
pub enum RegisterSize {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
    Bits128,
    Vector,
    Any,
    Unknown,
}

pub trait Registers {
    fn get_kind(&self, register: &str) -> RegisterKind;
    fn get_size(&self, register: &str) -> RegisterSize;
    fn is_sp(&self, register: &str) -> bool;
}

impl Display for RegisterSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                RegisterSize::Bits8 => "8",
                RegisterSize::Bits16 => "16",
                RegisterSize::Bits32 => "32",
                RegisterSize::Bits64 => "64",
                RegisterSize::Bits128 => "128",
                RegisterSize::Vector => "v",
                RegisterSize::Any => "a",
                RegisterSize::Unknown => "u",
            }
        )
    }
}

impl Display for RegisterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = Vec::new();
        if self.contains(RegisterKind::GENERAL_PURPOSE) {
            r.push("gp");
        }
        if self.contains(RegisterKind::FLOATING_POINT) {
            r.push("fp");
        }
        if self.contains(RegisterKind::SIMD) {
            r.push("simd");
        }
        if self.contains(RegisterKind::SCALABLE) {
            r.push("scale");
        }
        if self.contains(RegisterKind::PREDICATE) {
            r.push("pred");
        }
        if self.contains(RegisterKind::SP) {
            r.push("sp");
        }

        write!(f, "{}", r.join("|"))
    }
}
