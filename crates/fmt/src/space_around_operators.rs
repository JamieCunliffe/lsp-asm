use syntax::ast::{SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    if !options.space_around_operators {
        return root;
    }

    let replacements = root
        .descendants_with_tokens()
        .filter_map(|c| {
            c.into_token()
                .filter(|t| matches!(t.kind(), SyntaxKind::OPERATOR))
        })
        .flat_map(|child| [perform_after(child.clone()), perform_before(child)])
        .flatten()
        .collect::<Vec<_>>();

    perform_replacements(replacements);
    root
}

fn perform_after(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    let next = child.next_sibling_or_token()?;

    if let Some(ws) = next
        .into_token()
        .filter(|t| matches!(t.kind(), SyntaxKind::WHITESPACE))
    {
        if ws.text() == " " {
            None
        } else {
            Some((
                Position::Replace(ws),
                create_token(SyntaxKind::WHITESPACE, " "),
            ))
        }
    } else {
        Some((
            Position::After(child),
            create_token(SyntaxKind::WHITESPACE, " "),
        ))
    }
}

fn perform_before(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    // If there is nothing after the token then skip anything before
    // as the operator would be be incomplete.
    child.next_sibling_or_token()?;
    let prev = child.prev_sibling_or_token()?;

    if let Some(ws) = prev
        .into_token()
        .filter(|t| matches!(t.kind(), SyntaxKind::WHITESPACE))
    {
        if ws.text() == " " {
            None
        } else {
            Some((
                Position::Replace(ws),
                create_token(SyntaxKind::WHITESPACE, " "),
            ))
        }
    } else {
        Some((
            Position::Before(child),
            create_token(SyntaxKind::WHITESPACE, " "),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_minus_operator() {
        crate::format_test!(
r#".size main, .Lfunc_end0-main"# => r#".size main, .Lfunc_end0 - main"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_minus_operator_nothing_required() {
        crate::format_test!(
r#".size main, .Lfunc_end0 - main"# => r#".size main, .Lfunc_end0 - main"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_minus_operator_remove_spaces() {
        crate::format_test!(
r#".size main, .Lfunc_end0    -          main"# => r#".size main, .Lfunc_end0 - main"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_plus_operator() {
        crate::format_test!(
r#".size main, .Lfunc_end0+main"# => r#".size main, .Lfunc_end0 + main"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_plus_operator_with_literal() {
        crate::format_test!(
r#"adrp x8, data+8"# => r#"adrp x8, data + 8"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_negative_number() {
        crate::format_test!(
r#"mov x9, #-2"# => r#"mov x9, #-2"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_fmt_negative_incomplete() {
        crate::format_test!(
            r#"
    mov x9, #-
next:"#
=>
r#"
    mov x9, #-
next:"#,
            &Default::default(), perform_pass);
    }

    #[test]
    fn test_disabled() {
        let opts = FormatOptions {
            space_around_operators: false,
            ..Default::default()
        };

        crate::format_test!(
r#".size main, .Lfunc_end0-main"# => r#".size main, .Lfunc_end0-main"#,
            &opts, perform_pass);
    }
}
