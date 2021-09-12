use base::register::{RegisterKind, RegisterSize, Registers};
use base::Architecture;
use parser::config::ParserConfig;
use parser::Register;

pub struct AArch64 {}
impl Registers for AArch64 {
    fn get_kind(&self, name: &str) -> RegisterKind {
        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterKind::GENERAL_PURPOSE,
            "w" => RegisterKind::GENERAL_PURPOSE,
            "s" if name == "sp" => RegisterKind::SP,
            "q" => RegisterKind::FLOATING_POINT,
            "d" => RegisterKind::FLOATING_POINT,
            "s" if name != "sp" => RegisterKind::FLOATING_POINT,
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
        match name.get(0..=0).unwrap_or("\0") {
            "x" | "r" => RegisterSize::Bits64,
            "s" if name == "sp" => RegisterSize::Bits64,
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
        name == "sp"
    }
}

pub(crate) fn registers_for_architecture(arch: &Architecture) -> Option<impl Registers> {
    match arch {
        Architecture::AArch64 => Some(AArch64 {}),
        Architecture::X86_64 => None,
        Architecture::Unknown => None,
    }
}

/// Gets an index for this register that can be used for comparisons, the id
/// that is returned should only be considered valid for the given parser config
/// when comparing.
pub(crate) fn register_id(name: &str, config: &ParserConfig) -> Option<i8> {
    if let Some(registers) = config.registers {
        let name = parser::register_name(name);
        registers
            .iter()
            .enumerate()
            .find(|(_, register)| register.names.contains(&name))
            .map(|(idx, _)| idx as _)
    } else {
        None
    }
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

pub(crate) const AARCH64_REGISTERS: [Register; 81] = [
    Register::new(&["x0", "w0"]),
    Register::new(&["x1", "w1"]),
    Register::new(&["x2", "w2"]),
    Register::new(&["x3", "w3"]),
    Register::new(&["x4", "w4"]),
    Register::new(&["x5", "w5"]),
    Register::new(&["x6", "w6"]),
    Register::new(&["x7", "w7"]),
    Register::new(&["x8", "w8"]),
    Register::new(&["x9", "w9"]),
    Register::new(&["x10", "w10"]),
    Register::new(&["x11", "w11"]),
    Register::new(&["x12", "w12"]),
    Register::new(&["x13", "w13"]),
    Register::new(&["x14", "w14"]),
    Register::new(&["x15", "w15"]),
    Register::new(&["x16", "w16"]),
    Register::new(&["x17", "w17"]),
    Register::new(&["x18", "w18"]),
    Register::new(&["x19", "w19"]),
    Register::new(&["x20", "w20"]),
    Register::new(&["x21", "w21"]),
    Register::new(&["x22", "w22"]),
    Register::new(&["x23", "w23"]),
    Register::new(&["x24", "w24"]),
    Register::new(&["x25", "w25"]),
    Register::new(&["x26", "w26"]),
    Register::new(&["x27", "w27"]),
    Register::new(&["x28", "w28"]),
    Register::new(&["x29", "w29"]),
    Register::new(&["x30", "w30"]),
    Register::new(&["sp"]),
    Register::new(&["xzr", "wzr"]),
    Register::new(&[
        "d0", "s0", "q0", "h0", "b0", "v0", "v0.8b", "v0.16b", "v0.4h", "v0.8h", "v0.2s", "v0.4s",
        "v0.2d", "z0", "z0.b", "z0.h", "z0.s", "z0.d", "z0.q",
    ]),
    Register::new(&[
        "d1", "s1", "q1", "h1", "b1", "v1", "v1.8b", "v1.16b", "v1.4h", "v1.8h", "v1.2s", "v1.4s",
        "v1.2d", "z1", "z1.b", "z1.h", "z1.s", "z1.d", "z1.q",
    ]),
    Register::new(&[
        "d2", "s2", "q2", "h2", "b2", "v2", "v2.8b", "v2.16b", "v2.4h", "v2.8h", "v2.2s", "v2.4s",
        "v2.2d", "z2", "z2.b", "z2.h", "z2.s", "z2.d", "z2.q",
    ]),
    Register::new(&[
        "d3", "s3", "q3", "h3", "b3", "v3", "v3.8b", "v3.16b", "v3.4h", "v3.8h", "v3.2s", "v3.4s",
        "v3.2d", "z3", "z3.b", "z3.h", "z3.s", "z3.d", "z3.q",
    ]),
    Register::new(&[
        "d4", "s4", "q4", "h4", "b4", "v4", "v4.8b", "v4.16b", "v4.4h", "v4.8h", "v4.2s", "v4.4s",
        "v4.2d", "z4", "z4.b", "z4.h", "z4.s", "z4.d", "z4.q",
    ]),
    Register::new(&[
        "d5", "s5", "q5", "h5", "b5", "v5", "v5.8b", "v5.16b", "v5.4h", "v5.8h", "v5.2s", "v5.4s",
        "v5.2d", "z5", "z5.b", "z5.h", "z5.s", "z5.d", "z5.q",
    ]),
    Register::new(&[
        "d6", "s6", "q6", "h6", "b6", "v6", "v6.8b", "v6.16b", "v6.4h", "v6.8h", "v6.2s", "v6.4s",
        "v6.2d", "z6", "z6.b", "z6.h", "z6.s", "z6.d", "z6.q",
    ]),
    Register::new(&[
        "d7", "s7", "q7", "h7", "b7", "v7", "v7.8b", "v7.16b", "v7.4h", "v7.8h", "v7.2s", "v7.4s",
        "v7.2d", "z7", "z7.b", "z7.h", "z7.s", "z7.d", "z7.q",
    ]),
    Register::new(&[
        "d8", "s8", "q8", "h8", "b8", "v8", "v8.8b", "v8.16b", "v8.4h", "v8.8h", "v8.2s", "v8.4s",
        "v8.2d", "z8", "z8.b", "z8.h", "z8.s", "z8.d", "z8.q",
    ]),
    Register::new(&[
        "d9", "s9", "q9", "h9", "b9", "v9", "v9.8b", "v9.16b", "v9.4h", "v9.8h", "v9.2s", "v9.4s",
        "v9.2d", "z9", "z9.b", "z9.h", "z9.s", "z9.d", "z9.q",
    ]),
    Register::new(&[
        "d10", "s10", "q10", "h10", "b10", "v10", "v10.8b", "v10.16b", "v10.4h", "v10.8h",
        "v10.2s", "v10.4s", "v10.2d", "z10", "z10.b", "z10.h", "z10.s", "z10.d", "z10.q",
    ]),
    Register::new(&[
        "d11", "s11", "q11", "h11", "b11", "v11", "v11.8b", "v11.16b", "v11.4h", "v11.8h",
        "v11.2s", "v11.4s", "v11.2d", "z11", "z11.b", "z11.h", "z11.s", "z11.d", "z11.q",
    ]),
    Register::new(&[
        "d12", "s12", "q12", "h12", "b12", "v12", "v12.8b", "v12.16b", "v12.4h", "v12.8h",
        "v12.2s", "v12.4s", "v12.2d", "z12", "z12.b", "z12.h", "z12.s", "z12.d", "z12.q",
    ]),
    Register::new(&[
        "d13", "s13", "q13", "h13", "b13", "v13", "v13.8b", "v13.16b", "v13.4h", "v13.8h",
        "v13.2s", "v13.4s", "v13.2d", "z13", "z13.b", "z13.h", "z13.s", "z13.d", "z13.q",
    ]),
    Register::new(&[
        "d14", "s14", "q14", "h14", "b14", "v14", "v14.8b", "v14.16b", "v14.4h", "v14.8h",
        "v14.2s", "v14.4s", "v14.2d", "z14", "z14.b", "z14.h", "z14.s", "z14.d", "z14.q",
    ]),
    Register::new(&[
        "d15", "s15", "q15", "h15", "b15", "v15", "v15.8b", "v15.16b", "v15.4h", "v15.8h",
        "v15.2s", "v15.4s", "v15.2d", "z15", "z15.b", "z15.h", "z15.s", "z15.d", "z15.q",
    ]),
    Register::new(&[
        "d16", "s16", "q16", "h16", "b16", "v16", "v16.8b", "v16.16b", "v16.4h", "v16.8h",
        "v16.2s", "v16.4s", "v16.2d", "z16", "z16.b", "z16.h", "z16.s", "z16.d", "z16.q",
    ]),
    Register::new(&[
        "d17", "s17", "q17", "h17", "b17", "v17", "v17.8b", "v17.16b", "v17.4h", "v17.8h",
        "v17.2s", "v17.4s", "v17.2d", "z17", "z17.b", "z17.h", "z17.s", "z17.d", "z17.q",
    ]),
    Register::new(&[
        "d18", "s18", "q18", "h18", "b18", "v18", "v18.8b", "v18.16b", "v18.4h", "v18.8h",
        "v18.2s", "v18.4s", "v18.2d", "z18", "z18.b", "z18.h", "z18.s", "z18.d", "z18.q",
    ]),
    Register::new(&[
        "d19", "s19", "q19", "h19", "b19", "v19", "v19.8b", "v19.16b", "v19.4h", "v19.8h",
        "v19.2s", "v19.4s", "v19.2d", "z19", "z19.b", "z19.h", "z19.s", "z19.d", "z19.q",
    ]),
    Register::new(&[
        "d20", "s20", "q20", "h20", "b20", "v20", "v20.8b", "v20.16b", "v20.4h", "v20.8h",
        "v20.2s", "v20.4s", "v20.2d", "z20", "z20.b", "z20.h", "z20.s", "z20.d", "z20.q",
    ]),
    Register::new(&[
        "d21", "s21", "q21", "h21", "b21", "v21", "v21.8b", "v21.16b", "v21.4h", "v21.8h",
        "v21.2s", "v21.4s", "v21.2d", "z21", "z21.b", "z21.h", "z21.s", "z21.d", "z21.q",
    ]),
    Register::new(&[
        "d22", "s22", "q22", "h22", "b22", "v22", "v22.8b", "v22.16b", "v22.4h", "v22.8h",
        "v22.2s", "v22.4s", "v22.2d", "z22", "z22.b", "z22.h", "z22.s", "z22.d", "z22.q",
    ]),
    Register::new(&[
        "d23", "s23", "q23", "h23", "b23", "v23", "v23.8b", "v23.16b", "v23.4h", "v23.8h",
        "v23.2s", "v23.4s", "v23.2d", "z23", "z23.b", "z23.h", "z23.s", "z23.d", "z23.q",
    ]),
    Register::new(&[
        "d24", "s24", "q24", "h24", "b24", "v24", "v24.8b", "v24.16b", "v24.4h", "v24.8h",
        "v24.2s", "v24.4s", "v24.2d", "z24", "z24.b", "z24.h", "z24.s", "z24.d", "z24.q",
    ]),
    Register::new(&[
        "d25", "s25", "q25", "h25", "b25", "v25", "v25.8b", "v25.16b", "v25.4h", "v25.8h",
        "v25.2s", "v25.4s", "v25.2d", "z25", "z25.b", "z25.h", "z25.s", "z25.d", "z25.q",
    ]),
    Register::new(&[
        "d26", "s26", "q26", "h26", "b26", "v26", "v26.8b", "v26.16b", "v26.4h", "v26.8h",
        "v26.2s", "v26.4s", "v26.2d", "z26", "z26.b", "z26.h", "z26.s", "z26.d", "z26.q",
    ]),
    Register::new(&[
        "d27", "s27", "q27", "h27", "b27", "v27", "v27.8b", "v27.16b", "v27.4h", "v27.8h",
        "v27.2s", "v27.4s", "v27.2d", "z27", "z27.b", "z27.h", "z27.s", "z27.d", "z27.q",
    ]),
    Register::new(&[
        "d28", "s28", "q28", "h28", "b28", "v28", "v28.8b", "v28.16b", "v28.4h", "v28.8h",
        "v28.2s", "v28.4s", "v28.2d", "z28", "z28.b", "z28.h", "z28.s", "z28.d", "z28.q",
    ]),
    Register::new(&[
        "d29", "s29", "q29", "h29", "b29", "v29", "v29.8b", "v29.16b", "v29.4h", "v29.8h",
        "v29.2s", "v29.4s", "v29.2d", "z29", "z29.b", "z29.h", "z29.s", "z29.d", "z29.q",
    ]),
    Register::new(&[
        "d30", "s30", "q30", "h30", "b30", "v30", "v30.8b", "v30.16b", "v30.4h", "v30.8h",
        "v30.2s", "v30.4s", "v30.2d", "z30", "z30.b", "z30.h", "z30.s", "z30.d", "z30.q",
    ]),
    Register::new(&[
        "d31", "s31", "q31", "h31", "b31", "v31", "v31.8b", "v31.16b", "v31.4h", "v31.8h",
        "v31.2s", "v31.4s", "v31.2d", "z31", "z31.b", "z31.h", "z31.s", "z31.d", "z31.q",
    ]),
    Register::new(&[
        "p0", "p0/z", "p0/m", "p0.b", "p0.h", "p0.s", "p0.s", "p0.d", "p0.q",
    ]),
    Register::new(&[
        "p1", "p1/z", "p1/m", "p1.b", "p1.h", "p1.s", "p1.s", "p1.d", "p1.q",
    ]),
    Register::new(&[
        "p2", "p2/z", "p2/m", "p2.b", "p2.h", "p2.s", "p2.s", "p2.d", "p2.q",
    ]),
    Register::new(&[
        "p3", "p3/z", "p3/m", "p3.b", "p3.h", "p3.s", "p3.s", "p3.d", "p3.q",
    ]),
    Register::new(&[
        "p4", "p4/z", "p4/m", "p4.b", "p4.h", "p4.s", "p4.s", "p4.d", "p4.q",
    ]),
    Register::new(&[
        "p5", "p5/z", "p5/m", "p5.b", "p5.h", "p5.s", "p5.s", "p5.d", "p5.q",
    ]),
    Register::new(&[
        "p6", "p6/z", "p6/m", "p6.b", "p6.h", "p6.s", "p6.s", "p6.d", "p6.q",
    ]),
    Register::new(&[
        "p7", "p7/z", "p7/m", "p7.b", "p7.h", "p7.s", "p7.s", "p7.d", "p7.q",
    ]),
    Register::new(&[
        "p8", "p8/z", "p8/m", "p8.b", "p8.h", "p8.s", "p8.s", "p8.d", "p8.q",
    ]),
    Register::new(&[
        "p9", "p9/z", "p9/m", "p9.b", "p9.h", "p9.s", "p9.s", "p9.d", "p9.q",
    ]),
    Register::new(&[
        "p10", "p10/z", "p10/m", "p10.b", "p10.h", "p10.s", "p10.s", "p10.d", "p10.q",
    ]),
    Register::new(&[
        "p11", "p11/z", "p11/m", "p11.b", "p11.h", "p11.s", "p11.s", "p11.d", "p11.q",
    ]),
    Register::new(&[
        "p12", "p12/z", "p12/m", "p12.b", "p12.h", "p12.s", "p12.s", "p12.d", "p12.q",
    ]),
    Register::new(&[
        "p13", "p13/z", "p13/m", "p13.b", "p13.h", "p13.s", "p13.s", "p13.d", "p13.q",
    ]),
    Register::new(&[
        "p14", "p14/z", "p14/m", "p14.b", "p14.h", "p14.s", "p14.s", "p14.d", "p14.q",
    ]),
    Register::new(&[
        "p15", "p15/z", "p15/m", "p15.b", "p15.h", "p15.s", "p15.s", "p15.d", "p15.q",
    ]),
];
