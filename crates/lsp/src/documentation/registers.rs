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
                "Q" => RegisterKind::FLOATING_POINT,
                "D" => RegisterKind::FLOATING_POINT,
                "S" => RegisterKind::FLOATING_POINT,
                "H" => RegisterKind::FLOATING_POINT,
                "B" => RegisterKind::FLOATING_POINT,
                "V" => RegisterKind::SIMD,
                "Z" => RegisterKind::SCALABLE,
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
                "Q" => RegisterSize::Bits128(RegisterKind::FLOATING_POINT),
                "D" => RegisterSize::Bits64(RegisterKind::FLOATING_POINT),
                "S" => RegisterSize::Bits32(RegisterKind::FLOATING_POINT),
                "H" => RegisterSize::Bits16(RegisterKind::FLOATING_POINT),
                "B" => RegisterSize::Bits8(RegisterKind::FLOATING_POINT),
                "V" => RegisterSize::Vector,
                "Z" => RegisterSize::Scalable(RegisterKind::SCALABLE),
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
    "<Bt>",
    "<Da>",
    "<Dd>",
    "<Dm>",
    "<Dn>",
    "<Dt1>",
    "<Dt2>",
    "<Dt>",
    "<Ha>",
    "<Hd>",
    "<Hm>",
    "<Hn>",
    "<Ht>",
    "<Pd>",
    "<Pd>.<T>",
    "<Pd>.B",
    "<Pd>.D",
    "<Pd>.H",
    "<Pd>.Q",
    "<Pd>.S",
    "<Pd>/M",
    "<Pd>/Z",
    "<Pdm>",
    "<Pdm>.B",
    "<Pdm>.D",
    "<Pdm>.H",
    "<Pdm>.Q",
    "<Pdm>.S",
    "<Pdm>/M",
    "<Pdm>/Z",
    "<Pdn>",
    "<Pdn>.B",
    "<Pdn>.D",
    "<Pdn>.H",
    "<Pdn>.Q",
    "<Pdn>.S",
    "<Pdn>/M",
    "<Pdn>/Z",
    "<Pg>",
    "<Pg>.B",
    "<Pg>.D",
    "<Pg>.H",
    "<Pg>.Q",
    "<Pg>.S",
    "<Pg>/M",
    "<Pg>/Z",
    "<Pm>",
    "<Pm>.<T>",
    "<Pm>.B",
    "<Pm>.D",
    "<Pm>.H",
    "<Pm>.Q",
    "<Pm>.S",
    "<Pm>/M",
    "<Pm>/Z",
    "<Pn>",
    "<Pn>.<T>",
    "<Pn>.B",
    "<Pn>.D",
    "<Pn>.H",
    "<Pn>.Q",
    "<Pn>.S",
    "<Pn>/M",
    "<Pn>/Z",
    "<Pt>",
    "<Pt>.B",
    "<Pt>.D",
    "<Pt>.H",
    "<Pt>.Q",
    "<Pt>.S",
    "<Pt>/M",
    "<Pt>/Z",
    "<Qd>",
    "<Qn>",
    "<Qt1>",
    "<Qt2>",
    "<Qt>",
    "<R><m>",
    "<R><n>",
    "<Sa>",
    "<Sd>",
    "<Sm>",
    "<Sn>",
    "<St1>",
    "<St2>",
    "<St>",
    "<V>",
    "<Va>",
    "<Vb>",
    "<Vd>",
    "<Vd>.<T>",
    "<Vd>.<Tb>",
    "<Vd>.<Ts>",
    "<Vm>",
    "<Vm>.<T>",
    "<Vm>.<Ta>",
    "<Vm>.<Tb>",
    "<Vm>.<Ts>",
    "<VnADD1>",
    "<VnADD1>.16B",
    "<VnADD2>",
    "<VnADD2>.16B",
    "<VnADD3>",
    "<VnADD3>.16B",
    "<Vn>",
    "<Vn>.16B",
    "<Vn>.<T>",
    "<Vn>.<Ta>",
    "<Vn>.<Tb>",
    "<Vn>.<Ts>",
    "<Vt2>",
    "<Vt3>",
    "<Vt4>",
    "<Vt>",
    "<Vt>.<T>",
    "<W(sADD1)>",
    "<W(tADD1)>",
    "<Wa>",
    "<Wd>",
    "<Wdn>",
    "<Wd|WSP>",
    "<Wm>",
    "<Wn>",
    "<Wn|WSP>",
    "<Ws>",
    "<Wt1>",
    "<Wt2>",
    "<Wt>",
    "<X(sADD1)>",
    "<X(tADD1)>",
    "<Xa>",
    "<Xd>",
    "<Xdn>",
    "<Xd|SP>",
    "<Xm>",
    "<Xm|SP>",
    "<Xn>",
    "<Xn|SP>",
    "<Xs>",
    "<Xt1>",
    "<Xt2>",
    "<Xt>",
    "<Xt|SP>",
    "<ZM>",
    "<Za>",
    "<Zd>",
    "<Zd>.<T>",
    "<Zda>",
    "<Zda>.S",
    "<Zdn>",
    "<Zdn>.<T>",
    "<Zm>",
    "<Zm>.<T>",
    "<Zm>.D",
    "<Zm>.H",
    "<Zn>",
    "<Zn>.<T>",
    "<Zn>.H",
    "<Zt1>",
    "<Zt2>",
    "<Zt3>",
    "<Zt4>",
    "<Zt>",
    "<Zt>.<T>",
    "<Zt>.B",
    "<Zt>.D",
    "<Zt>.H",
    "<Zt>.S",
    "<Vd>.<Ta>",
])];