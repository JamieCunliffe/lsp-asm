use base::register::{RegisterKind, RegisterSize};

/// Mappings of register names used in documentation to their register size/kind.
/// This should be sorted so that the more specific names are first, the names are
/// replaced in the order they are contained in the list.
pub(crate) const REGISTER_REPLACEMENTS: &[(&str, &RegisterSize, &RegisterKind)] = &[
    ("<Vn+1>.16B", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn+2>.16B", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn+3>.16B", &RegisterSize::Vector, &RegisterKind::SIMD),
    (
        "<W(s+1)>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<W(t+1)>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<X(s+1)>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<X(t+1)>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    ("<Vd>.<Ta>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>.<Tb>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>.<Ts>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>.16B",  &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>.4S", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>.2D", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Va>.4S", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Va>.16B",  &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.<Ta>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.<Tb>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.<Ts>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.16B",  &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.4S", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.4B", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.2D", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.2H", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.8H", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.H", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.<Ta>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.<Tb>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.<Ts>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt2>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt3>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt4>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Zdn>.<T>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Pm>.<T>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.<T>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Vd>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.16B", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.8H", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.4S", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.2D", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn+1>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn+2>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn+3>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt>.<T>", &RegisterSize::Vector, &RegisterKind::SIMD),
    (
        "<Wd|WSP>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wn|WSP>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    ("<Zd>.<T>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zm>.<T>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>.<T>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>.<T>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Pd>.<T>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdm>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Xd|SP>", &RegisterSize::Bits64, &RegisterKind::GP_OR_SP),
    ("<Xm|SP>", &RegisterSize::Bits64, &RegisterKind::GP_OR_SP),
    ("<Xn|SP>", &RegisterSize::Bits64, &RegisterKind::GP_OR_SP),
    ("<Xt|SP>", &RegisterSize::Bits64, &RegisterKind::GP_OR_SP),
    ("<Zda>.S", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Pd>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pd>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>.B", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>.D", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>.H", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>.Q", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>.S", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>/M", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>/Z", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<R><m>", &RegisterSize::Any, &RegisterKind::GENERAL_PURPOSE),
    ("<R><n>", &RegisterSize::Any, &RegisterKind::GENERAL_PURPOSE),
    ("<Zm>.B", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zm>.D", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zm>.H", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zm>.S", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>.B", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>.D", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>.H", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>.S", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>.B", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>.D", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>.H", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>.S", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    (
        "<Dt1>",
        &RegisterSize::Bits64,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<Dt2>",
        &RegisterSize::Bits64,
        &RegisterKind::FLOATING_POINT,
    ),
    ("<Pdm>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pdn>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    (
        "<Qt1>",
        &RegisterSize::Bits128,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<Qt2>",
        &RegisterSize::Bits128,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<St1>",
        &RegisterSize::Bits32,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<St2>",
        &RegisterSize::Bits32,
        &RegisterKind::FLOATING_POINT,
    ),
    ("<Vt2>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt3>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt4>", &RegisterSize::Vector, &RegisterKind::SIMD),
    (
        "<Wdn>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wt1>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wt2>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xdn>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xt1>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xt2>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    ("<Zda>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zdn>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt1>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt2>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt3>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt4>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Bt>", &RegisterSize::Bits8, &RegisterKind::FLOATING_POINT),
    ("<Da>", &RegisterSize::Bits64, &RegisterKind::FLOATING_POINT),
    ("<Dd>", &RegisterSize::Bits64, &RegisterKind::FLOATING_POINT),
    ("<Dm>", &RegisterSize::Bits64, &RegisterKind::FLOATING_POINT),
    ("<Dn>", &RegisterSize::Bits64, &RegisterKind::FLOATING_POINT),
    ("<Dt>", &RegisterSize::Bits64, &RegisterKind::FLOATING_POINT),
    ("<Ha>", &RegisterSize::Bits16, &RegisterKind::FLOATING_POINT),
    ("<Hd>", &RegisterSize::Bits16, &RegisterKind::FLOATING_POINT),
    ("<Hm>", &RegisterSize::Bits16, &RegisterKind::FLOATING_POINT),
    ("<Hn>", &RegisterSize::Bits16, &RegisterKind::FLOATING_POINT),
    ("<Ht>", &RegisterSize::Bits16, &RegisterKind::FLOATING_POINT),
    ("<Pd>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pg>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pm>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pn>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    ("<Pt>", &RegisterSize::Vector, &RegisterKind::PREDICATE),
    (
        "<Qd>",
        &RegisterSize::Bits128,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<Qn>",
        &RegisterSize::Bits128,
        &RegisterKind::FLOATING_POINT,
    ),
    (
        "<Qt>",
        &RegisterSize::Bits128,
        &RegisterKind::FLOATING_POINT,
    ),
    ("<Sa>", &RegisterSize::Bits32, &RegisterKind::FLOATING_POINT),
    ("<Sd>", &RegisterSize::Bits32, &RegisterKind::FLOATING_POINT),
    ("<Sm>", &RegisterSize::Bits32, &RegisterKind::FLOATING_POINT),
    ("<Sn>", &RegisterSize::Bits32, &RegisterKind::FLOATING_POINT),
    ("<St>", &RegisterSize::Bits32, &RegisterKind::FLOATING_POINT),
    ("<Va>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vb>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vd>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vm>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vn>", &RegisterSize::Vector, &RegisterKind::SIMD),
    ("<Vt>", &RegisterSize::Vector, &RegisterKind::SIMD),
    (
        "<Wa>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wd>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wm>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wn>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Ws>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Wt>",
        &RegisterSize::Bits32,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xa>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xd>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xm>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xn>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xs>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    (
        "<Xt>",
        &RegisterSize::Bits64,
        &RegisterKind::GENERAL_PURPOSE,
    ),
    ("<ZM>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Za>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zd>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zm>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zn>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<Zt>", &RegisterSize::Vector, &RegisterKind::SCALABLE),
    ("<V>", &RegisterSize::Vector, &RegisterKind::SIMD),
];

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    fn known_register_names() -> Vec<String> {
        let mut names = super::REGISTER_REPLACEMENTS
            .iter()
            .map(|(_, s, k)| documentation::registers::to_documentation_name(k, s))
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    #[test]
    fn ensure_known_registers_match() {
        let parser_registers = documentation::registers::DOCUMENTATION_REGISTERS
            .keys()
            .cloned()
            .map(|x| x.to_string())
            .sorted()
            .collect::<Vec<_>>();
        let registers = known_register_names();

        if registers != parser_registers {
            eprintln!("Registers don't match, does documentation::registers::DOCUMENTATION_REGISTERS or documentation_builder::register_replacements::REGISTER_REPLACEMENTS need updating?");
        }

        assert_eq!(registers, parser_registers)
    }
}
