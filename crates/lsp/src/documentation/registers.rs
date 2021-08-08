#![allow(clippy::upper_case_acronyms)]
use crate::asm::ast::SyntaxToken;
use crate::asm::registers::{Register, RegisterKind, RegisterSize, Registers};

pub(super) struct DocRegisters {}

impl Registers for DocRegisters {
    fn get_kind(&self, token: &SyntaxToken) -> RegisterKind {
        let name = token.text();
        if name.get(0..=0) == Some("<") {
            match name.get(1..=1).unwrap_or("\0") {
                "X" => RegisterKind::GENERAL_PURPOSE,
                "W" => RegisterKind::GENERAL_PURPOSE,
                "Q" => RegisterKind::SIMD_FP,
                "D" => RegisterKind::SIMD_FP,
                "S" => RegisterKind::SIMD_FP,
                "H" => RegisterKind::SIMD_FP,
                "B" => RegisterKind::SIMD_FP,
                "Z" => RegisterKind::SCALABLE_SIMD,
                "P" => RegisterKind::PREDICATE,
                _ => RegisterKind::NONE,
            }
        } else {
            RegisterKind::NONE
        }
    }

    fn get_size(&self, token: &SyntaxToken) -> RegisterSize {
        let name = token.text();
        if name.get(0..=0) == Some("<") {
            match name.get(1..=1).unwrap_or("\0") {
                "X" => RegisterSize::Bits64(RegisterKind::GENERAL_PURPOSE),
                "W" => RegisterSize::Bits32(RegisterKind::GENERAL_PURPOSE),
                "Q" => RegisterSize::Bits128(RegisterKind::SIMD_FP),
                "D" => RegisterSize::Bits64(RegisterKind::SIMD_FP),
                "S" => RegisterSize::Bits32(RegisterKind::SIMD_FP),
                "H" => RegisterSize::Bits16(RegisterKind::SIMD_FP),
                "B" => RegisterSize::Bits8(RegisterKind::SIMD_FP),
                "Z" => RegisterSize::Scalable(RegisterKind::SCALABLE_SIMD),
                "P" => RegisterSize::Scalable(RegisterKind::PREDICATE),
                _ => RegisterSize::Unknown,
            }
        } else {
            RegisterSize::Unknown
        }
    }

    fn is_sp(&self, token: &SyntaxToken) -> bool {
        token.text().contains("SP")
    }
}

pub(super) const DOC_REGISTERS: DocRegisters = DocRegisters {};

pub(super) const DOCUMENTATION_REGISTERS: [Register; 1] = [Register::new(&[
    "<Xt>", "<Wt>", "<Xt1>", "<Wt1>", "<Xt2>", "<Wt2>", "<Xn|SP>", "<Bt>", "<Da>", "<Dd>", "<Dm>",
    "<Dn>", "<Dt1>", "<Dt2>", "<Dt>", "<Ha>", "<Hd>", "<Hm>", "<Hn>", "<Ht>", "<Pd>", "<Pdm>",
    "<Pdn>", "<Pg>", "<Pm>", "<Pn>", "<Pt>", "<Qd>", "<Qn>", "<Qt1>", "<Qt2>", "<Qt>", "<Sa>",
    "<Sd>", "<Sm>", "<Sn>", "<St1>", "<St2>", "<St>", "<V>", "<Va>", "<Vb>", "<Vd>", "<Vm>",
    "<Vn+1>", "<Vn+2>", "<Vn+3>", "<Vn>", "<Vt2>", "<Vt3>", "<Vt4>", "<Vt>", "<W(s+1)>",
    "<W(t+1)>", "<Wa>", "<Wd>", "<Wdn>", "<Wd|WSP>", "<Wm>", "<Wn>", "<Wn|WSP>", "<Ws>", "<Wt1>",
    "<Wt2>", "<Wt>", "<X(s+1)>", "<X(t+1)>", "<Xa>", "<Xd>", "<Xdn>", "<Xd|SP>", "<Xm>", "<Xm|SP>",
    "<Xn>", "<Xn|SP>", "<Xs>", "<Xt1>", "<Xt2>", "<Xt>", "<Xt|SP>", "<ZM>", "<Za>", "<Zd>",
    "<Zda>", "<Zdn>", "<Zm>", "<Zn>", "<Zt1>", "<Zt2>", "<Zt3>", "<Zt4>", "<Zt>",
])];
