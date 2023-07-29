mod lsp_asm_arch_directive;

use base::Architecture;
use rowan::TextSize;
use syntax::ast::SyntaxNode;

macro_rules! define_actions {
    ($(($arch:expr, $name:ident)),*) => {
        const ALL_ACTIONS: &[(base::Architecture, RunActionFn)] = &[
            $(
                (
                    $arch,
                    $name::run,
                )
            ),*
        ];

        #[cfg(test)]
        const TEST_ALL_ACTIONS: &[(base::Architecture, &'static str)] = &[
            $(
                (
                    $arch,
                    stringify!($name),
                )
            ),*
        ];
    };
}
type RunActionFn = fn(&ActionContext) -> Option<Vec<CodeAction>>;

define_actions!(
    (Architecture::AArch64, lsp_asm_arch_directive),
    (Architecture::X86_64, lsp_asm_arch_directive),
    (Architecture::Unknown, lsp_asm_arch_directive)
);

#[derive(Debug, Default)]
pub struct CodeAction {
    pub name: String,
    pub edit: Vec<Edit>,
}

#[derive(Debug, Default)]
pub struct Edit {
    pub start: TextSize,
    pub end: TextSize,
    pub text: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ActionContext {
    pub arch: Architecture,
    pub root: SyntaxNode,
    pub start: TextSize,
    pub end: TextSize,
}

pub fn code_actions(
    arch: Architecture,
    root: SyntaxNode,
    start: TextSize,
    end: TextSize,
) -> Vec<CodeAction> {
    let ctx = ActionContext {
        arch,
        root,
        start,
        end,
    };

    ALL_ACTIONS
        .iter()
        .flat_map(|(req_arch, action)| {
            if arch == *req_arch {
                action(&ctx)
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_all_arch_actions() {
        macro_rules! all_arch {
            ($($name:ident),*) => {
                $(
                    assert_eq!(
                        TEST_ALL_ACTIONS
                            .iter()
                            .filter(|(_, action)| *action == stringify!($name))
                            .count(),
                        Architecture::iter().count(),
                        concat!("The codeaction: `", stringify!($name), "` isn't implemented for all architectures"));
                )*
            }
        }

        all_arch!(lsp_asm_arch_directive);
    }
}
