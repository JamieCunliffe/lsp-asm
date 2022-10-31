use syntax::ast::{SyntaxKind, SyntaxNode};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    let indentation = options.make_indentation();

    let replacements = root
        .descendants_with_tokens()
        .filter_map(|c| {
            c.into_token().filter(|t| {
                matches!(t.kind(), SyntaxKind::MNEMONIC)
                    && matches!(
                        t.parent().map(|n| n.kind()),
                        Some(SyntaxKind::INSTRUCTION) | Some(SyntaxKind::DIRECTIVE)
                    )
            })
        })
        .filter_map(|child| {
            if let Some(ws) = child.prev_token() {
                let text = ws.text().trim_end_matches(|c| c == '\t' || c == ' ');
                let new_text = format!("{text}{indentation}");

                if new_text == ws.text() {
                    None
                } else {
                    Some((
                        Position::Replace(ws),
                        create_token(SyntaxKind::WHITESPACE, &new_text),
                    ))
                }
            } else {
                Some((
                    Position::FirstChild(child.parent()?.parent()?),
                    create_token(SyntaxKind::WHITESPACE, &indentation),
                ))
            }
        })
        .collect::<Vec<_>>();

    perform_replacements(replacements);

    root
}

#[cfg(test)]
mod tests {
    use crate::TabKind;

    use super::*;

    #[test]
    fn test_indent_instruction() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!("ADD x0, x0, x1" => "    ADD x0, x0, x1", &opts, perform_pass);
    }

    #[test]
    fn test_partial_indent_instruction() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!("  ADD x0, x0, x1" => "    ADD x0, x0, x1", &opts, perform_pass);
    }

    #[test]
    fn test_too_much_indent_instruction() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!("     ADD x0, x0, x1" => "    ADD x0, x0, x1", &opts, perform_pass);
    }

    #[test]
    fn test_indent_instruction_in_label() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!("entry:\nADD x0, x0, x1" => "entry:\n    ADD x0, x0, x1", &opts, perform_pass);
    }

    #[test]
    fn test_indent_directive() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!(".p2align 2" => "    .p2align 2", &opts, perform_pass);
    }

    #[test]
    fn test_mixed_space_tab_expect_space() {
        let opts = FormatOptions {
            indentation_spaces: 4,
            ..Default::default()
        };
        crate::format_test!("
 .p2align 2
\t.p2align 2
"
=>
 "
    .p2align 2
    .p2align 2
", &opts, perform_pass);
    }

    #[test]
    fn test_mixed_space_tab_expect_tab() {
        let opts = FormatOptions {
            tab_kind: TabKind::Tab,
            ..Default::default()
        };
        crate::format_test!("
 .p2align 2
\t .p2align 2
"
=>
"
\t.p2align 2
\t.p2align 2
", &opts, perform_pass);
    }
}
