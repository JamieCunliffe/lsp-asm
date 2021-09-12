use base::{register::Registers, Architecture};
use documentation::registers::DOC_REGISTERS;
use itertools::Itertools;
use parser::Register;

use crate::asm::registers::{AARCH64_REGISTERS, X86_64_REGISTERS};
use crate::types::{CompletionItem, CompletionKind};

pub(super) fn complete_registers(
    token: &str,
    registers: &Option<impl Registers>,
    architecture: &Architecture,
) -> Vec<CompletionItem> {
    let register_names: &[Register] = match architecture {
        Architecture::X86_64 => &X86_64_REGISTERS,
        Architecture::AArch64 => &AARCH64_REGISTERS,
        Architecture::Unknown => return Default::default(),
    };

    let register_kind = DOC_REGISTERS.get_kind(token);
    let register_size = DOC_REGISTERS.get_size(token);

    register_names
        .iter()
        .flat_map(|r| r.names)
        .filter(|r| {
            if let Some(registers) = registers {
                registers.get_size(r) == register_size
                    && register_kind.contains(registers.get_kind(r))
            } else {
                true
            }
        })
        .map(|r| CompletionItem {
            text: r.to_string(),
            details: String::from(""),
            documentation: None,
            kind: CompletionKind::Register,
        })
        .collect_vec()
}

#[cfg(test)]
mod tests {

    use std::iter;

    use crate::asm::registers::registers_for_architecture;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_aarch64_x_registers() {
        let register = "<GP_64>";
        let expected = (0..=30)
            .map(|x| CompletionItem {
                text: format!("x{}", x),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Register,
            })
            .chain(iter::once(CompletionItem {
                text: "xzr".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Register,
            }))
            .collect_vec();

        assert_eq!(
            complete_registers(
                register,
                &registers_for_architecture(&Architecture::AArch64),
                &Architecture::AArch64,
            ),
            expected,
        );
    }

    #[test]
    fn test_aarch64_x_inc_sp_registers() {
        let register = "<GP|SP_64>";
        let expected = (0..=30)
            .map(|x| CompletionItem {
                text: format!("x{}", x),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Register,
            })
            .chain(iter::once(CompletionItem {
                text: "sp".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Register,
            }))
            .chain(iter::once(CompletionItem {
                text: "xzr".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Register,
            }))
            .collect_vec();

        assert_eq!(
            complete_registers(
                register,
                &registers_for_architecture(&Architecture::AArch64),
                &Architecture::AArch64,
            ),
            expected,
        );
    }
}
