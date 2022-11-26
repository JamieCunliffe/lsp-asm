use syntax::ast::{SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    let replacements = root
        .descendants_with_tokens()
        .filter_map(|c| {
            c.into_token()
                .filter(|t| matches!(t.kind(), SyntaxKind::COMMA))
        })
        .flat_map(|child| {
            [
                if options.space_after_comma {
                    perform_after(child.clone())
                } else {
                    remove_after(child.clone())
                },
                if options.space_before_comma {
                    perform_before(child)
                } else {
                    remove_before(child)
                },
            ]
        })
        .flatten()
        .collect::<Vec<_>>();

    perform_replacements(replacements);

    root
}

fn perform_after(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    if let Some(ws) = child
        .next_sibling_or_token()?
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
    if let Some(ws) = child
        .prev_sibling_or_token()?
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

fn remove_after(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    child
        .next_sibling_or_token()?
        .into_token()
        .filter(|t| matches!(t.kind(), SyntaxKind::WHITESPACE))
        .map(|ws| (Position::Replace(ws), create_token(SyntaxKind::ROOT, "")))
}

fn remove_before(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    child
        .prev_sibling_or_token()?
        .into_token()
        .filter(|t| matches!(t.kind(), SyntaxKind::WHITESPACE))
        .map(|ws| (Position::Replace(ws), create_token(SyntaxKind::ROOT, "")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma_after() {
        crate::format_test!("ADD x1,x1,x3" => "ADD x1, x1, x3", &Default::default(), perform_pass);
        crate::format_test!("ADD x1, x1,  x3" => "ADD x1, x1, x3", &Default::default(), perform_pass);
    }

    #[test]
    fn test_comma_before_after() {
        let opts = FormatOptions {
            space_before_comma: true,
            ..Default::default()
        };
        crate::format_test!("ADD x1,x1,x3" => "ADD x1 , x1 , x3", &opts, perform_pass);
        crate::format_test!("ADD x1 ,x1    ,x3" => "ADD x1 , x1 , x3", &opts, perform_pass);
    }

    #[test]
    fn test_no_comma_before_after() {
        let opts = FormatOptions {
            space_before_comma: false,
            space_after_comma: false,
            ..Default::default()
        };
        crate::format_test!("ADD x1 , x1, x3" => "ADD x1,x1,x3", &opts, perform_pass);
        crate::format_test!("ADD x1 ,x1    ,x3" => "ADD x1,x1,x3", &opts, perform_pass);
    }

    #[test]
    fn test_incomplete_trailing_comma() {
        let opts = FormatOptions {
            space_after_comma: true,
            ..Default::default()
        };
        crate::format_test!(
"ADD x1, x1,
main:"
=>
"ADD x1, x1,
main:", &opts, perform_pass);
    }
}
