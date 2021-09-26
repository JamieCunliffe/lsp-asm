use base::register::{RegisterKind, RegisterSize, Registers};
use base::Architecture;
use documentation::registers::DOC_REGISTERS;
use itertools::Itertools;
use parser::Register;
use syntax::alias::Alias;

use crate::asm::registers::{AARCH64_REGISTERS, X86_64_REGISTERS};
use crate::types::{CompletionItem, CompletionKind};

pub(super) fn complete_registers(
    token: &str,
    registers: &dyn Registers,
    architecture: &Architecture,
    alias: &Alias,
) -> Vec<CompletionItem> {
    let register_names: &[Register] = match architecture {
        Architecture::X86_64 => &X86_64_REGISTERS,
        Architecture::AArch64 => &AARCH64_REGISTERS,
        Architecture::Unknown => return Default::default(),
    };

    let register_kind = DOC_REGISTERS.get_kind(token);
    let register_size = DOC_REGISTERS.get_size(token);

    let mut completions = register_names
        .iter()
        .flat_map(|r| r.names)
        .filter(|r| {
            registers.get_size(r) == register_size && register_kind.contains(registers.get_kind(r))
        })
        .map(to_completion)
        .collect_vec();

    completions.extend(get_aliases(alias, register_kind, register_size, registers));

    completions
}

fn get_aliases<'a>(
    alias: &'a Alias,
    kind: RegisterKind,
    size: RegisterSize,
    registers: &'a (dyn Registers + 'a),
) -> impl Iterator<Item = CompletionItem> + 'a {
    alias
        .get_alias_for_kind_size(kind, size, registers)
        .map(to_completion)
}

fn to_completion<T>(register: T) -> CompletionItem
where
    T: ToString,
{
    CompletionItem {
        text: register.to_string(),
        details: String::from(""),
        documentation: None,
        kind: CompletionKind::Register,
    }
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
                registers_for_architecture(&Architecture::AArch64),
                &Architecture::AArch64,
                &Default::default(),
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
                registers_for_architecture(&Architecture::AArch64),
                &Architecture::AArch64,
                &Default::default()
            ),
            expected,
        );
    }
}
