use base::register::{RegisterKind, RegisterSize};

macro_rules! build_registers {
    ($size:expr, $kind:expr, $name:literal) => (
        ($name, &$size, &$kind)
    );
    ($(($size:expr, $kind:expr) => [ $($name:literal),* $(,)?];)*) => {
        &[
            $($(build_registers!($size, $kind, $name)),*),*
        ]
    };
}

/// Mappings of register names used in documentation to their register size/kind.
/// This should be sorted so that the more specific names are first, the names are
/// replaced in the order they are contained in the list.
pub(crate) const REGISTER_REPLACEMENTS: &[(&str, &RegisterSize, &RegisterKind)] = build_registers! {
    (RegisterSize::Vector, RegisterKind::SIMD) => [
        "<Vn+1>.16B", "<Vn+2>.16B", "<Vn+3>.16B", "<Vd>.<Ta>", "<Vd>.<Tb>",
        "<Vd>.<Ts>", "<Vd>.16B", "<Vd>.4S", "<Vd>.2D", "<Va>.4S",
        "<Va>.16B", "<Va><d>", "<Va><n>", "<Vb><d>", "<Vb><m>", "<Vb><n>",
        "<Vm>.<Ta>", "<Vm>.<Tb>", "<Vm>.<Ts>", "<Vm>.16B", "<Vm>.4S",
        "<Vm>.4B", "<Vm>.2D", "<Vm>.2H", "<Vm>.8H", "<Vm>.H", "<Vn>.<Ta>",
        "<Vn>.<Tb>", "<Vn>.<Ts>", "<Vt2>.<T>", "<Vt2>.B", "<Vt2>.D",
        "<Vt2>.H", "<Vt2>.S", "<Vt3>.<T>", "<Vt3>.B", "<Vt3>.D",
        "<Vt3>.H", "<Vt3>.S", "<Vt4>.<T>", "<Vt4>.B", "<Vt4>.D",
        "<Vt4>.H", "<Vt4>.S", "<Vt>.B", "<Vt>.D", "<Vt>.H", "<Vt>.S",
        "<Vd>.<T>", "<Vm>.<T>", "<Vn>.16B", "<Vn>.8H", "<Vn>.4S",
        "<Vn>.2D", "<Vn>.<T>", "<Vn+1>", "<Vn+2>", "<Vn+3>", "<Vt>.<T>",
        "<Vt2>", "<Vt3>", "<Vt4>", "<Va>", "<Vb>", "<Vd>", "<Vm>", "<Vn>",
        "<Vt>", "<V><n>", "<V><d>", "<V>"
    ];
    (RegisterSize::Vector, RegisterKind::SCALABLE) => [
        "<Zdn>.<T>", "<Zdn>.B", "<Zdn1>.<T>", "<Zdn1>.H", "<Zdn2>.<T>",
        "<Zdn2>.H", "<Zdn4>.<T>", "<Zdn4>.H", "<Zdn>.D", "<Zdn>.H", "<Zdn>.S",
        "<Zd1>.<T>", "<Zd1>.B", "<Zd1>.D", "<Zd1>.H", "<Zd1>.Q", "<Zd1>.S",
        "<Zd2>.<T>", "<Zd2>.B", "<Zd2>.D", "<Zd2>.H", "<Zd2>.Q", "<Zd2>.S",
        "<Zd3>.<T>", "<Zd3>.H", "<Zd4>.<T>", "<Zd4>.B", "<Zd4>.D", "<Zd4>.H",
        "<Zd4>.Q", "<Zd4>.S", "<Zd>.<T>", "<Zd>.Q", "<Zd>.B", "<Zd>.D",
        "<Zd>.H", "<Zd>.S", "<Zm>.<T>", "<Zn>.<Tb>", "<Zn>.<T>", "<Zt>.<T>",
        "<Zda>.<T>", "<Zda>.S", "<Zda>.D", "<Zda>.H", "<Zm>.B", "<Zm>.D",
        "<Zm>.H", "<Zm>.S", "<Zm>.Q", "<Zk>.D", "<Zm1>.<T>", "<Zm1>.<Tb>",
        "<Zm1>.B", "<Zm1>.H", "<Zm2>.<T>", "<Zm2>.<Tb>", "<Zm2>.B", "<Zm2>.H",
        "<Zm4>.<T>", "<Zm4>.<Tb>", "<Zm4>.B", "<Zm4>.H", "<Zm>.D]",
        "<Zn1>.<T>", "<Zn1>.<Tb>", "<Zn1>.B", "<Zn1>.D", "<Zn1>.H", "<Zn1>.Q",
        "<Zn1>.S", "<Zn2>.<T>", "<Zn2>.<Tb>", "<Zn2>.B", "<Zn2>.D", "<Zn2>.H",
        "<Zn2>.S", "<Zn4>.<T>", "<Zn4>.<Tb>", "<Zn4>.B", "<Zn4>.D", "<Zn4>.H",
        "<Zn4>.Q", "<Zn4>.S", "<Zt>.Q", "<Zn>.B", "<Zn>.D", "<Zn>.H",
        "<Zn>.S", "<Zn>.Q", "<Zt>.B", "<Zt>.D", "<Zt>.H", "<Zt>.S", "<Zda>",
        "<Zdn>", "<Zt1>.B", "<Zt1>.D", "<Zt1>.H", "<Zt1>.Q", "<Zt1>.S",
        "<Zt2>.B", "<Zt2>.D", "<Zt2>.H", "<Zt2>.Q", "<Zt2>.S", "<Zt3>.B",
        "<Zt3>.D", "<Zt3>.H", "<Zt3>.Q", "<Zt3>.S", "<Zt4>.B", "<Zt4>.D",
        "<Zt4>.H", "<Zt4>.Q", "<Zt4>.S", "<Zt1>", "<Zt2>", "<Zt3>", "<Zt4>",
        "<Za>.<T>", "<Za>", "<Zd>", "<Zm>.<Tb>", "<Zm>", "<Zn>", "<Zt>"
    ];
    (RegisterSize::Vector, RegisterKind::PREDICATE) => [
        "<Pd>.<T>", "<Pdm>.B", "<Pdm>.D", "<Pdm>.H", "<Pdm>.Q", "<Pdm>.S",
        "<Pdm>/M", "<Pdm>/Z", "<Pdn>.B", "<Pdn>.D", "<Pdn>.H", "<Pdn>.Q",
        "<Pdn>.S", "<Pdn>/M", "<Pdn>/Z", "<Pd>.B", "<Pd>.D", "<Pd>.H",
        "<Pd>.Q", "<Pd>.S", "<Pd>/M", "<Pd>/Z", "<Pd1>.<T>", "<Pd2>.<T>",
        "<Pg>/<ZM>", "<Pg>.B", "<Pg>.D", "<Pg>.H", "<Pg>.Q", "<Pg>.S",
        "<Pg>/M", "<Pg>/Z", "<Pm>.B", "<Pm>.D", "<Pm>.H", "<Pm>.Q", "<Pm>.S",
        "<Pm>/M", "<Pm>/Z", "<Pm>.<T>", "<Pn>.<T>", "<PNd>.<T>", "<PNg>/Z",
        "<PNn>.<T>", "<PNn>", "<PNg>", "<Pn>.B", "<Pn>.D", "<Pn>.H", "<Pn>.Q",
        "<Pn>.S", "<Pn>/M", "<Pn>/Z", "<Pt>.B", "<Pt>.D", "<Pt>.H", "<Pt>.Q",
        "<Pt>.S", "<Pt>/M", "<Pt>/Z", "<Pv>/M", "<Pv>", "<Pdm>", "<Pdn>.<T>",
        "<Pdn>", "<Pd>", "<Pg>", "<Pm>", "<Pn>", "<Pt>"
    ];
    (RegisterSize::Bits32, RegisterKind::GENERAL_PURPOSE) => [
        "<W(s+1)>", "<W(t+1)>", "<Wd|WSP>", "<Wn|WSP>", "<Wdn>", "<Wt1>",
        "<Wt2>", "<Wa>", "<Wd>", "<Wm>", "<Wn>", "<Ws>", "<Wt>",
    ];
    (RegisterSize::Any, RegisterKind::GENERAL_PURPOSE) => [
        "<R><m>",  "<R><n>", "<R><t>", "<R><dn>", "<R><d>"
    ];
    (RegisterSize::Any, RegisterKind::GP_OR_SP) => [
        "<R><n|SP>"
    ];
    (RegisterSize::Bits64, RegisterKind::GENERAL_PURPOSE) => [
        "<X(s+1)>", "<X(t+1)>", "<Xdn>", "<Xt1>", "<Xt2>", "<Xa>", "<Xd>",
        "<Xm>", "<Xn>", "<Xs>", "<Xt>"
    ];
    (RegisterSize::Bits64, RegisterKind::GP_OR_SP) => [
        "<Xd|SP>", "<Xm|SP>", "<Xn|SP>", "<Xt|SP>"
    ];
    (RegisterSize::Bits8, RegisterKind::FLOATING_POINT) => [
        "<Bt>"
    ];
    (RegisterSize::Bits16, RegisterKind::FLOATING_POINT) => [
        "<Ha>", "<Hd>", "<Hm>", "<Hn>", "<Ht>"
    ];
    (RegisterSize::Bits64, RegisterKind::FLOATING_POINT) => [
        "<Da>", "<Dd>", "<Dm>", "<Dn>", "<Dt1>", "<Dt2>","<Dt>",
    ];
    (RegisterSize::Bits32, RegisterKind::FLOATING_POINT) => [
        "<Sa>", "<Sd>", "<Sm>", "<Sn>", "<St1>", "<St2>", "<St>"
    ];
    (RegisterSize::Bits128, RegisterKind::FLOATING_POINT) => [
        "<Qd>", "<Qn>", "<Qt1>", "<Qt2>", "<Qt>"
    ];
};

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
