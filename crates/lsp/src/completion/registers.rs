use arch::registers::RegisterList;
use base::register::{RegisterKind, RegisterSize, Registers};
use documentation::registers::DOC_REGISTERS;
use itertools::Itertools;
use syntax::alias::Alias;

use crate::completion::CompletionContext;
use crate::types::{CompletionItem, CompletionKind};

pub(super) fn complete_registers(
    to_complete: &str,
    context: &CompletionContext,
) -> Vec<CompletionItem> {
    let registers = context.registers;

    let register_names = RegisterList::from_architecture(context.architecture());

    let register_kind = DOC_REGISTERS.get_kind(to_complete);
    let register_size = DOC_REGISTERS.get_size(to_complete);

    let mut completions = register_names
        .names()
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
    use base::Architecture;
    use pretty_assertions::assert_eq;
    use syntax::ast::{find_kind_index, SyntaxKind};

    #[test]
    fn test_aarch64_x_registers() {
        let register = "<gp_64>";
        let expected = (0..=30)
            .map(|x| CompletionItem {
                text: format!("x{x}"),
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
            .sorted()
            .collect_vec();
        let parser = Parser::in_memory(
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
                text: format!("x{x}"),
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
            .sorted()
            .collect_vec();

        let parser = Parser::in_memory(
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
