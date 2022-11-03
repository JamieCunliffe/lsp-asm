use syntax::ast::{find_kind_index, SyntaxKind, SyntaxNode};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    if !options.align_first_operand {
        return root;
    }

    let instructions = root
        .descendants()
        .filter_map(|n| {
            if matches!(n.kind(), SyntaxKind::INSTRUCTION) {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let max = instructions.iter().fold(0, |max, node| {
        find_kind_index(node, 0, SyntaxKind::MNEMONIC)
            .and_then(|t| t.as_token().map(|t| t.text().len()))
            .unwrap_or(0)
            .max(max)
    });

    let replacements = instructions
        .iter()
        .filter_map(|node| {
            let token =
                find_kind_index(node, 0, SyntaxKind::MNEMONIC).and_then(|t| t.into_token())?;
            let num_spaces = max - token.text().len();
            let spaces = " ".repeat(num_spaces + 1);
            let maybe_ws = token.next_token()?;
            if maybe_ws.text() != spaces && matches!(maybe_ws.kind(), SyntaxKind::WHITESPACE) {
                Some((
                    Position::Replace(maybe_ws),
                    create_token(SyntaxKind::WHITESPACE, &spaces),
                ))
            } else {
                None
            }
        })
        .collect();

    perform_replacements(replacements);
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_operands() {
        let options = FormatOptions {
            align_first_operand: true,
            ..Default::default()
        };
        crate::format_test!(
            r#"
a x1
aa x1
aaa x1
aaaa x1
aaaaaa x1
"# => r#"
a      x1
aa     x1
aaa    x1
aaaa   x1
aaaaaa x1
"#, &options, perform_pass);
    }

    #[test]
    fn test_align_operands_already_aligned() {
        let options = FormatOptions {
            align_first_operand: true,
            ..Default::default()
        };
        crate::format_test!(
            r#"
a      x1
aa     x1
aaaaaa x1
aaa    x1
aaaa   x1
"# => r#"
a      x1
aa     x1
aaaaaa x1
aaa    x1
aaaa   x1
"#, &options, perform_pass);
    }

    #[test]
    fn test_align_indented() {
        let options = FormatOptions {
            align_first_operand: true,
            ..Default::default()
        };
        crate::format_test!(
            r#"
a x1
aa x1
label:
    aaa x1
    aaaa x1
    aaaaaa x1
"# => r#"
a      x1
aa     x1
label:
    aaa    x1
    aaaa   x1
    aaaaaa x1
"#, &options, perform_pass);
    }

    #[test]
    fn test_no_align_operands() {
        let options = FormatOptions {
            align_first_operand: false,
            ..Default::default()
        };
        crate::format_test!(
            r#"
a x1
aa  x1
aaa x1
aaaa x1
aaaaaa x1
"# => r#"
a x1
aa  x1
aaa x1
aaaa x1
aaaaaa x1
"#, &options, perform_pass);
    }
}
