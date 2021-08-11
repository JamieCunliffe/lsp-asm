use crate::types::Architecture;

use super::ast::SyntaxToken;
use super::config::ParserConfig;
use bitflags::bitflags;

/// A register
#[derive(Debug, PartialEq)]
pub struct Register {
    /// A list of names that for this register, each name in this list is
    /// considered to be the same hardware register.
    names: &'static [&'static str],
}

impl Register {
    pub const fn new(names: &'static [&'static str]) -> Self {
        Self { names }
    }
}

bitflags! {
    pub struct RegisterKind : u32 {
        const NONE            = 0b00000000;
        const GENERAL_PURPOSE = 0b00000001;
        const FLOATING_POINT  = 0b00000010;
        const SIMD            = 0b00000100;
        const SCALABLE        = 0b00001000;
        const PREDICATE       = 0b00010000;

        const SIMD_FP         = Self::FLOATING_POINT.bits | Self::SIMD.bits;
        const SCALABLE_SIMD   = Self::SIMD.bits | Self::SCALABLE.bits;
    }
}

#[derive(Debug, PartialEq)]
pub enum RegisterSize {
    Bits8(RegisterKind),
    Bits16(RegisterKind),
    Bits32(RegisterKind),
    Bits64(RegisterKind),
    Bits128(RegisterKind),
    Scalable(RegisterKind),
    Unknown,
}

pub(crate) trait Registers {
    fn get_kind(&self, token: &SyntaxToken) -> RegisterKind;
    fn get_size(&self, token: &SyntaxToken) -> RegisterSize;
    fn is_sp(&self, token: &SyntaxToken) -> bool;
}

pub struct AArch64 {}
impl Registers for AArch64 {
    fn get_kind(&self, token: &SyntaxToken) -> RegisterKind {
        let name = register_name(token.text());
        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterKind::GENERAL_PURPOSE,
            "w" => RegisterKind::GENERAL_PURPOSE,
            "q" => RegisterKind::SIMD_FP,
            "d" => RegisterKind::SIMD_FP,
            "s" if token.text() != "sp" => RegisterKind::SIMD_FP,
            "h" => RegisterKind::SIMD_FP,
            "b" => RegisterKind::SIMD_FP,
            "v" => {
                let size = name.split('.').next().unwrap_or("\0");
                match size {
                    "8b" | "16b" => RegisterKind::SIMD_FP,
                    "4h" | "8h" => RegisterKind::SIMD_FP,
                    "2s" | "4s" => RegisterKind::SIMD_FP,
                    "2d" => RegisterKind::SIMD_FP,
                    _ => RegisterKind::SIMD_FP,
                }
            }
            _ => RegisterKind::NONE,
        }
    }

    fn get_size(&self, token: &SyntaxToken) -> RegisterSize {
        let name = register_name(token.text());
        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterSize::Bits64(RegisterKind::GENERAL_PURPOSE),
            "w" => RegisterSize::Bits32(RegisterKind::GENERAL_PURPOSE),
            "q" => RegisterSize::Bits128(RegisterKind::SIMD_FP),
            "d" => RegisterSize::Bits64(RegisterKind::SIMD_FP),
            "s" => RegisterSize::Bits32(RegisterKind::SIMD_FP),
            "h" => RegisterSize::Bits16(RegisterKind::SIMD_FP),
            "b" => RegisterSize::Bits8(RegisterKind::SIMD_FP),
            "v" => {
                let size = name.split('.').next().unwrap_or("\0");
                match size {
                    "8b" | "16b" => RegisterSize::Bits8(RegisterKind::SIMD_FP),
                    "4h" | "8h" => RegisterSize::Bits16(RegisterKind::SIMD_FP),
                    "2s" | "4s" => RegisterSize::Bits32(RegisterKind::SIMD_FP),
                    "2d" => RegisterSize::Bits64(RegisterKind::SIMD_FP),
                    _ => RegisterSize::Bits128(RegisterKind::SIMD_FP),
                }
            }
            _ => RegisterSize::Unknown,
        }
    }

    fn is_sp(&self, token: &SyntaxToken) -> bool {
        register_name(token.text()) == "sp"
    }
}

pub(crate) fn registers_for_architecture(arch: &Architecture) -> Option<impl Registers> {
    match arch {
        Architecture::AArch64 => Some(AArch64 {}),
        Architecture::X86_64 => None,
        Architecture::Unknown => None,
    }
}

/// Determine if `name` is a valid register
pub(crate) fn is_register(name: &str, config: &ParserConfig) -> bool {
    if let Some(registers) = config.registers {
        let name = register_name(name);
        registers
            .iter()
            .any(|register| register.names.contains(&name))
    } else {
        false
    }
}

/// Gets an index for this register that can be used for comparisons, the id
/// that is returned should only be considered valid for the given parser config
/// when comparing.
pub(crate) fn register_id(name: &str, config: &ParserConfig) -> Option<i8> {
    if let Some(registers) = config.registers {
        let name = register_name(name);
        registers
            .iter()
            .enumerate()
            .find(|(_, register)| register.names.contains(&name))
            .map(|(idx, _)| idx as _)
    } else {
        None
    }
}

fn register_name(name: &str) -> &str {
    name.strip_prefix('%').unwrap_or(name)
}

pub(crate) const X86_64_REGISTERS: [Register; 9] = [
    Register::new(&["rax", "eax", "ax", "ah", "al"]),
    Register::new(&["rbx", "ebx", "bx", "bh", "bl"]),
    Register::new(&["rcx", "ecx", "cx", "ch", "cl"]),
    Register::new(&["rdx", "edx", "dx", "dh", "dl"]),
    Register::new(&["rbp"]),
    Register::new(&["rsp"]),
    Register::new(&["rsi"]),
    Register::new(&["rdi", "edi"]),
    Register::new(&["rip"]),
];

pub(crate) const AARCH64_REGISTERS: [Register; 65] = [
    Register::new(&["r0", "x0", "w0"]),
    Register::new(&["r1", "x1", "w1"]),
    Register::new(&["r2", "x2", "w2"]),
    Register::new(&["r3", "x3", "w3"]),
    Register::new(&["r4", "x4", "w4"]),
    Register::new(&["r5", "x5", "w5"]),
    Register::new(&["r6", "x6", "w6"]),
    Register::new(&["r7", "x7", "w7"]),
    Register::new(&["r8", "x8", "w8"]),
    Register::new(&["r9", "x9", "w9"]),
    Register::new(&["r10", "x10", "w10"]),
    Register::new(&["r11", "x11", "w11"]),
    Register::new(&["r12", "x12", "w12"]),
    Register::new(&["r13", "x13", "w13"]),
    Register::new(&["r14", "x14", "w14"]),
    Register::new(&["r15", "x15", "w15"]),
    Register::new(&["r16", "x16", "w16"]),
    Register::new(&["r17", "x17", "w17"]),
    Register::new(&["r18", "x18", "w18"]),
    Register::new(&["r19", "x19", "w19"]),
    Register::new(&["r20", "x20", "w20"]),
    Register::new(&["r21", "x21", "w21"]),
    Register::new(&["r22", "x22", "w22"]),
    Register::new(&["r23", "x23", "w23"]),
    Register::new(&["r24", "x24", "w24"]),
    Register::new(&["r25", "x25", "w25"]),
    Register::new(&["r26", "x26", "w26"]),
    Register::new(&["r27", "x27", "w27"]),
    Register::new(&["r28", "x28", "w28"]),
    Register::new(&["r29", "x29", "w29"]),
    Register::new(&["r30", "x30", "w30"]),
    Register::new(&["sp"]),
    Register::new(&["xzr", "wzr"]),
    Register::new(&[
        "v0", "v0.8b", "v0.16b", "v0.4h", "v0.8h", "v0.2s", "v0.4s", "v0.2d", "d0", "s0", "q0",
        "h0", "b0",
    ]),
    Register::new(&[
        "v1", "v1.8b", "v1.16b", "v1.4h", "v1.8h", "v1.2s", "v1.4s", "v1.2d", "d1", "s1", "q1",
        "h1", "b1",
    ]),
    Register::new(&[
        "v2", "v2.8b", "v2.16b", "v2.4h", "v2.8h", "v2.2s", "v2.4s", "v2.2d", "d2", "s2", "q2",
        "h2", "b2",
    ]),
    Register::new(&[
        "v3", "v3.8b", "v3.16b", "v3.4h", "v3.8h", "v3.2s", "v3.4s", "v3.2d", "d3", "s3", "q3",
        "h3", "b3",
    ]),
    Register::new(&[
        "v4", "v4.8b", "v4.16b", "v4.4h", "v4.8h", "v4.2s", "v4.4s", "v4.2d", "d4", "s4", "q4",
        "h4", "b4",
    ]),
    Register::new(&[
        "v5", "v5.8b", "v5.16b", "v5.4h", "v5.8h", "v5.2s", "v5.4s", "v5.2d", "d5", "s5", "q5",
        "h5", "b5",
    ]),
    Register::new(&[
        "v6", "v6.8b", "v6.16b", "v6.4h", "v6.8h", "v6.2s", "v6.4s", "v6.2d", "d6", "s6", "q6",
        "h6", "b6",
    ]),
    Register::new(&[
        "v7", "v7.8b", "v7.16b", "v7.4h", "v7.8h", "v7.2s", "v7.4s", "v7.2d", "d7", "s7", "q7",
        "h7", "b7",
    ]),
    Register::new(&[
        "v8", "v8.8b", "v8.16b", "v8.4h", "v8.8h", "v8.2s", "v8.4s", "v8.2d", "d8", "s8", "q8",
        "h8", "b8",
    ]),
    Register::new(&[
        "v9", "v9.8b", "v9.16b", "v9.4h", "v9.8h", "v9.2s", "v9.4s", "v9.2d", "d9", "s9", "q9",
        "h9", "b9",
    ]),
    Register::new(&[
        "v10", "v10.8b", "v10.16b", "v10.4h", "v10.8h", "v10.2s", "v10.4s", "v10.2d", "d10", "s10",
        "q10", "h10", "b10",
    ]),
    Register::new(&[
        "v11", "v11.8b", "v11.16b", "v11.4h", "v11.8h", "v11.2s", "v11.4s", "v11.2d", "d11", "s11",
        "q11", "h11", "b11",
    ]),
    Register::new(&[
        "v12", "v12.8b", "v12.16b", "v12.4h", "v12.8h", "v12.2s", "v12.4s", "v12.2d", "d12", "s12",
        "q12", "h12", "b12",
    ]),
    Register::new(&[
        "v13", "v13.8b", "v13.16b", "v13.4h", "v13.8h", "v13.2s", "v13.4s", "v13.2d", "d13", "s13",
        "q13", "h13", "b13",
    ]),
    Register::new(&[
        "v14", "v14.8b", "v14.16b", "v14.4h", "v14.8h", "v14.2s", "v14.4s", "v14.2d", "d14", "s14",
        "q14", "h14", "b14",
    ]),
    Register::new(&[
        "v15", "v15.8b", "v15.16b", "v15.4h", "v15.8h", "v15.2s", "v15.4s", "v15.2d", "d15", "s15",
        "q15", "h15", "b15",
    ]),
    Register::new(&[
        "v16", "v16.8b", "v16.16b", "v16.4h", "v16.8h", "v16.2s", "v16.4s", "v16.2d", "d16", "s16",
        "q16", "h16", "b16",
    ]),
    Register::new(&[
        "v17", "v17.8b", "v17.16b", "v17.4h", "v17.8h", "v17.2s", "v17.4s", "v17.2d", "d17", "s17",
        "q17", "h17", "b17",
    ]),
    Register::new(&[
        "v18", "v18.8b", "v18.16b", "v18.4h", "v18.8h", "v18.2s", "v18.4s", "v18.2d", "d18", "s18",
        "q18", "h18", "b18",
    ]),
    Register::new(&[
        "v19", "v19.8b", "v19.16b", "v19.4h", "v19.8h", "v19.2s", "v19.4s", "v19.2d", "d19", "s19",
        "q19", "h19", "b19",
    ]),
    Register::new(&[
        "v20", "v20.8b", "v20.16b", "v20.4h", "v20.8h", "v20.2s", "v20.4s", "v20.2d", "d20", "s20",
        "q20", "h20", "b20",
    ]),
    Register::new(&[
        "v21", "v21.8b", "v21.16b", "v21.4h", "v21.8h", "v21.2s", "v21.4s", "v21.2d", "d21", "s21",
        "q21", "h21", "b21",
    ]),
    Register::new(&[
        "v22", "v22.8b", "v22.16b", "v22.4h", "v22.8h", "v22.2s", "v22.4s", "v22.2d", "d22", "s22",
        "q22", "h22", "b22",
    ]),
    Register::new(&[
        "v23", "v23.8b", "v23.16b", "v23.4h", "v23.8h", "v23.2s", "v23.4s", "v23.2d", "d23", "s23",
        "q23", "h23", "b23",
    ]),
    Register::new(&[
        "v24", "v24.8b", "v24.16b", "v24.4h", "v24.8h", "v24.2s", "v24.4s", "v24.2d", "d24", "s24",
        "q24", "h24", "b24",
    ]),
    Register::new(&[
        "v25", "v25.8b", "v25.16b", "v25.4h", "v25.8h", "v25.2s", "v25.4s", "v25.2d", "d25", "s25",
        "q25", "h25", "b25",
    ]),
    Register::new(&[
        "v26", "v26.8b", "v26.16b", "v26.4h", "v26.8h", "v26.2s", "v26.4s", "v26.2d", "d26", "s26",
        "q26", "h26", "b26",
    ]),
    Register::new(&[
        "v27", "v27.8b", "v27.16b", "v27.4h", "v27.8h", "v27.2s", "v27.4s", "v27.2d", "d27", "s27",
        "q27", "h27", "b27",
    ]),
    Register::new(&[
        "v28", "v28.8b", "v28.16b", "v28.4h", "v28.8h", "v28.2s", "v28.4s", "v28.2d", "d28", "s28",
        "q28", "h28", "b28",
    ]),
    Register::new(&[
        "v29", "v29.8b", "v29.16b", "v29.4h", "v29.8h", "v29.2s", "v29.4s", "v29.2d", "d29", "s29",
        "q29", "h29", "b29",
    ]),
    Register::new(&[
        "v30", "v30.8b", "v30.16b", "v30.4h", "v30.8h", "v30.2s", "v30.4s", "v30.2d", "d30", "s30",
        "q30", "h30", "b30",
    ]),
    Register::new(&[
        "v31", "v31.8b", "v31.16b", "v31.4h", "v31.8h", "v31.2s", "v31.4s", "v31.2d", "d31", "s31",
        "q31", "h31", "b31",
    ]),
];
