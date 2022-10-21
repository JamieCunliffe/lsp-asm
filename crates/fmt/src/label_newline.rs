use syntax::ast::{SyntaxKind, SyntaxNode};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    if !options.newline_after_label {
        return root;
    }

    let indentation = options.make_indentation();
    let replacements = root
        .descendants_with_tokens()
        .filter_map(|d| {
            d.into_token()
                .filter(|d| matches!(d.kind(), SyntaxKind::LABEL))
        })
        .filter_map(|t| {
            if let Some(ws) = t
                .next_token()
                .filter(|next| matches!(next.kind(), SyntaxKind::WHITESPACE))
            {
                if ws.text().contains('\n') {
                    None
                } else if matches!(
                    ws.next_sibling_or_token().map(|d| d.kind()),
                    Some(SyntaxKind::INSTRUCTION) | Some(SyntaxKind::DIRECTIVE)
                ) {
                    Some((
                        Position::Replace(ws),
                        create_token(SyntaxKind::WHITESPACE, &format!("\n{}", indentation)),
                    ))
                } else {
                    None
                }
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
    fn test_new_line() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!("label: ADD x1, x1, x2" => "label:\n    ADD x1, x1, x2", &opts, perform_pass);
    }

    #[test]
    fn test_new_line_directive() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!("label: .word 7" => "label:\n    .word 7", &opts, perform_pass);
    }

    #[test]
    fn test_new_line_not_required() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!("label:\n ADD x1, x1, x2" => "label:\n ADD x1, x1, x2", &opts, perform_pass);
    }

    #[test]
    fn test_no_instruction_after() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!("label: " => "label: ", &opts, perform_pass);
        crate::format_test!("label:" => "label:", &opts, perform_pass);
    }

    #[test]
    fn test_new_line_local() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!(".label: ADD x1, x1, x2" => ".label:\n    ADD x1, x1, x2", &opts, perform_pass);
    }

    #[test]
    fn test_no_new_line_comment() {
        let opts = FormatOptions {
            ..Default::default()
        };
        crate::format_test!("label: // Comment" => "label: // Comment", &opts, perform_pass);
    }

    #[test]
    fn test_no_new_line() {
        let opts = FormatOptions {
            newline_after_label: false,
            ..Default::default()
        };
        crate::format_test!("label: ADD x1, x1, x2" => "label: ADD x1, x1, x2", &opts, perform_pass);
    }
}
