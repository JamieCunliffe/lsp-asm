use syntax::ast::{find_kind_index, SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    if !(options.align_const_defs || options.align_register_alias) {
        return root;
    }

    let const_defs = build_node_group(root.descendants(), options);

    let replacements = const_defs
        .iter()
        .flat_map(|const_defs| {
            let max = const_defs.iter().fold(0, |max, node| {
                if matches!(node.kind(), SyntaxKind::CONST_DEF) {
                    find_kind_index(node, 0, SyntaxKind::NAME)
                } else {
                    find_kind_index(node, 0, SyntaxKind::REGISTER_ALIAS)
                }
                .and_then(|t| t.as_token().map(|t| t.text().len()))
                .unwrap_or(0)
                .max(max)
            });

            let max_mnemonic = const_defs.iter().fold(0, |max, node| {
                find_kind_index(node, 0, SyntaxKind::MNEMONIC)
                    .and_then(|t| t.as_token().map(|t| t.text().len()))
                    .unwrap_or(0)
                    .max(max)
            });

            const_defs
                .iter()
                .flat_map(move |node| {
                    [
                        handle_name(max, max_mnemonic, node),
                        match node.kind() {
                            SyntaxKind::CONST_DEF => handle_expr(node),
                            SyntaxKind::ALIAS => handle_register(node),
                            _ => None,
                        },
                    ]
                })
                .flatten()
        })
        .collect::<Vec<_>>();

    perform_replacements(replacements);

    root
}

fn build_node_group(
    mut iter: impl Iterator<Item = SyntaxNode>,
    options: &FormatOptions,
) -> Vec<Vec<SyntaxNode>> {
    let mut res: Vec<Vec<SyntaxNode>> = Default::default();
    let iter = iter.by_ref();

    while let Some(next) = iter.next() {
        if matches!(next.kind(), SyntaxKind::CONST_DEF | SyntaxKind::ALIAS) {
            res.push(
                std::iter::once(next)
                    .chain(
                        iter.take_while(|node| {
                            matches!(
                                node.kind(),
                                SyntaxKind::CONST_DEF | SyntaxKind::EXPR | SyntaxKind::ALIAS
                            )
                        })
                        .filter(|node| {
                            matches!(node.kind(), SyntaxKind::CONST_DEF | SyntaxKind::ALIAS)
                        }),
                    )
                    .filter(|node| {
                        node.first_token()
                            .map(|t| t.kind() != SyntaxKind::MNEMONIC)
                            .unwrap_or(false)
                    })
                    .filter(|node| match node.kind() {
                        SyntaxKind::CONST_DEF if !options.align_const_defs => false,
                        SyntaxKind::ALIAS if !options.align_register_alias => false,
                        _ => true,
                    })
                    .collect::<Vec<_>>(),
            )
        }
    }
    res
}

fn handle_name(
    max: usize,
    max_mnemonic: usize,
    node: &SyntaxNode,
) -> Option<(Position, SyntaxToken)> {
    let token = if matches!(node.kind(), SyntaxKind::CONST_DEF) {
        find_kind_index(node, 0, SyntaxKind::NAME)
    } else {
        find_kind_index(node, 0, SyntaxKind::REGISTER_ALIAS)
    }
    .and_then(|t| t.into_token())?;

    let mnemonic_len = find_kind_index(node, 0, SyntaxKind::MNEMONIC)
        .and_then(|t| t.as_token().map(|t| t.text().len()))
        .unwrap_or(0);

    let num_spaces = max - token.text().len();
    let spaces = " ".repeat(num_spaces + 1 + (max_mnemonic - mnemonic_len));
    let maybe_ws = token.next_sibling_or_token()?.into_token()?;

    if maybe_ws.text() != spaces && matches!(maybe_ws.kind(), SyntaxKind::WHITESPACE) {
        Some((
            Position::Replace(maybe_ws),
            create_token(SyntaxKind::WHITESPACE, &spaces),
        ))
    } else {
        None
    }
}

fn handle_expr(node: &SyntaxNode) -> Option<(Position, SyntaxToken)> {
    let expr = find_kind_index(node, 0, SyntaxKind::EXPR).and_then(|t| t.into_node())?;
    let spaces = " ";
    let maybe_ws = expr.first_child_or_token().map(|e| e.into_token())??;

    if maybe_ws.text() != spaces && matches!(maybe_ws.kind(), SyntaxKind::WHITESPACE) {
        Some((
            Position::Replace(maybe_ws),
            create_token(SyntaxKind::WHITESPACE, spaces),
        ))
    } else {
        None
    }
}

fn handle_register(node: &SyntaxNode) -> Option<(Position, SyntaxToken)> {
    let register = find_kind_index(node, 0, SyntaxKind::REGISTER).and_then(|t| t.into_token())?;
    let spaces = " ";
    let maybe_ws = register.prev_sibling_or_token()?.into_token()?;

    if maybe_ws.text() != spaces && matches!(maybe_ws.kind(), SyntaxKind::WHITESPACE) {
        Some((
            Position::Replace(maybe_ws),
            create_token(SyntaxKind::WHITESPACE, spaces),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_simple() {
        crate::format_test!(
r#"
FIRST .EQU 0x1
SECOND .EQU 0x2
"#
=>
r#"
FIRST  .EQU 0x1
SECOND .EQU 0x2
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_with_blocks() {
        crate::format_test!(
r#"
FIRST .EQU 0x1
SECOND .EQU 0x2
ADD x0, x1, x1
ANOTHER_DEF .EQU 0x3
NEXT .EQU 0x4
"#
=>
r#"
FIRST  .EQU 0x1
SECOND .EQU 0x2
ADD x0, x1, x1
ANOTHER_DEF .EQU 0x3
NEXT        .EQU 0x4
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_expr() {
        crate::format_test!(
r#"
FIRST .EQU         0x1
SECOND .EQU   0x2
ADD x0, x1, x1
"#
=>
r#"
FIRST  .EQU 0x1
SECOND .EQU 0x2
ADD x0, x1, x1
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_expr_different_equ() {
        crate::format_test!(
r#"
FIRST .EQU         0x1
SECOND EQU   0x2
ADD x0, x1, x1
"#
=>
r#"
FIRST  .EQU 0x1
SECOND  EQU 0x2
ADD x0, x1, x1
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_no_dot_equ() {
        crate::format_test!(
r#"
FIRST EQU         0x1
SECOND EQU   0x2
ADD x0, x1, x1
"#
=>
r#"
FIRST  EQU 0x1
SECOND EQU 0x2
ADD x0, x1, x1
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_reg_alias() {
        crate::format_test!(
r#"
test .req         x1
another .req   x2
onemore .req x3
ADD x0, x1, x1
"#
=>
r#"
test    .req x1
another .req x2
onemore .req x3
ADD x0, x1, x1
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_const_def_and_reg_alias() {
        crate::format_test!(
r#"
test .req         x1
another .req   x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#
=>
r#"
test    .req x1
another .req x2
onemore .req x3
adef    .equ 0x1
smaller  equ 0x2
next    .equ 0x8
ADD x0, x1, x1
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_const_def_incomplete() {
        crate::format_test!(
r#"
test  .equ
"#
=>
r#"
test .equ
"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_aligns_const_def_as_syntax_untouched() {
        crate::format_test!(
            ".equ label_add_64, label+64" => ".equ label_add_64, label+64",
            &Default::default(),
            perform_pass
        );
        crate::format_test!(
            ".equ    label_add_64,      label+64" => ".equ    label_add_64,      label+64",
            &Default::default(),
            perform_pass
        );
    }

    #[test]
    fn test_fmt_aligns_const_def_and_reg_alias_alias_disabled() {
        let options = FormatOptions {
            align_register_alias: false,
            ..Default::default()
        };

        crate::format_test!(
r#"
test .req         x1
another .req   x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#
=>
r#"
test .req         x1
another .req   x2
onemore .req x3
adef    .equ 0x1
smaller  equ 0x2
next    .equ 0x8
ADD x0, x1, x1
"#,
            &options, perform_pass);
    }

    #[test]
    fn test_fmt_aligns_const_def_and_reg_alias_const_disabled() {
        let options = FormatOptions {
            align_const_defs: false,
            ..Default::default()
        };

        crate::format_test!(
r#"
test .req         x1
another .req   x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#
=>
r#"
test    .req x1
another .req x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#,
            &options, perform_pass);
    }

    #[test]
    fn test_fmt_aligns_const_def_and_reg_alias_disabled() {
        let options = FormatOptions {
            align_const_defs: false,
            align_register_alias: false,
            ..Default::default()
        };

        crate::format_test!(
r#"
test .req         x1
another .req   x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#
=>
r#"
test .req         x1
another .req   x2
onemore .req x3
adef .equ 0x1
smaller   equ 0x2
next .equ       0x8
ADD x0, x1, x1
"#,
            &options, perform_pass);
    }
}
