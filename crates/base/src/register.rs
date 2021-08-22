use bitflags::bitflags;

bitflags! {
    pub struct RegisterKind : u32 {
        const NONE            = 0b00000000;
        const GENERAL_PURPOSE = 0b00000001;
        const FLOATING_POINT  = 0b00000010;
        const SIMD            = 0b00000100;
        const SCALABLE        = 0b00001000;
        const PREDICATE       = 0b00010000;
    }
}

#[derive(Debug, PartialEq)]
pub enum RegisterSize {
    Bits8(RegisterKind),
    Bits16(RegisterKind),
    Bits32(RegisterKind),
    Bits64(RegisterKind),
    Bits128(RegisterKind),
    Vector,
    Scalable(RegisterKind),
    Unknown,
}

pub trait Registers {
    fn get_kind(&self, register: &str) -> RegisterKind;
    fn get_size(&self, register: &str) -> RegisterSize;
    fn is_sp(&self, register: &str) -> bool;
}
