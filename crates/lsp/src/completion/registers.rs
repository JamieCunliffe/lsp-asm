use base::register::{RegisterKind, RegisterSize, Registers};
use base::Architecture;
use documentation::registers::DOC_REGISTERS;
use itertools::Itertools;
use parser::Register;
use syntax::alias::Alias;

use crate::asm::registers::{AARCH64_REGISTERS, X86_64_REGISTERS};
use crate::types::{CompletionItem, CompletionKind};

use super::CompletionContext;

pub(super) fn complete_registers(
    to_complete: &str,
    context: &CompletionContext,
) -> Vec<CompletionItem> {
    let architecture = context.architecture();
    let registers = context.registers;

    let register_names: &[Register] = match architecture {
        Architecture::AArch64 => &AARCH64_REGISTERS,
        Architecture::X86_64 => &X86_64_REGISTERS,
        Architecture::Unknown => return Default::default(),
    };

    let register_kind = DOC_REGISTERS.get_kind(to_complete);
    let register_size = DOC_REGISTERS.get_size(to_complete);

    let mut completions = register_names
        .iter()
        .flat_map(|r| r.names)
        .filter(|r| {
            registers.get_size(r) == register_size && register_kind.contains(registers.get_kind(r))
        })
        .map(to_completion)
        .collect_vec();

    completions.extend(get_aliases(
        context.alias(),
        register_kind,
        register_size,
        registers,
    ));

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

    use crate::asm::parser::Parser;
    use crate::config::LSPConfig;

    use super::*;
    use pretty_assertions::assert_eq;
    use syntax::ast::{find_kind_index, SyntaxKind};

    #[test]
    fn test_aarch64_x_registers() {
        let register = "<gp_64>";
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
        let parser = Parser::from(
            " ",
            &LSPConfig {
                architecture: Architecture::AArch64,
                ..Default::default()
            },
        );
        let context = CompletionContext::new(
            &parser,
            find_kind_index(&parser.tree(), 0, SyntaxKind::WHITESPACE)
                .unwrap()
                .into_token()
                .unwrap(),
            Default::default(),
        );
        assert_eq!(complete_registers(register, &context), expected,);
    }

    #[test]
    fn test_aarch64_x_inc_sp_registers() {
        let register = "<gp|sp_64>";
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

        let parser = Parser::from(
            " ",
            &LSPConfig {
                architecture: Architecture::AArch64,
                ..Default::default()
            },
        );
        let context = CompletionContext::new(
            &parser,
            find_kind_index(&parser.tree(), 0, SyntaxKind::WHITESPACE)
                .unwrap()
                .into_token()
                .unwrap(),
            Default::default(),
        );
        assert_eq!(complete_registers(register, &context), expected,);
    }
}
