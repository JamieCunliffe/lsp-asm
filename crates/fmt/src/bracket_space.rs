use syntax::ast::{SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::edit::{create_token, perform_replacements, Position};

use crate::FormatOptions;

pub(crate) fn perform_pass(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    let replacements = root
        .descendants_with_tokens()
        .filter_map(|c| {
            c.into_token().filter(|t| {
                matches!(
                    t.kind(),
                    SyntaxKind::L_CURLY
                        | SyntaxKind::R_CURLY
                        | SyntaxKind::L_SQ
                        | SyntaxKind::R_SQ
                        | SyntaxKind::L_PAREN
                        | SyntaxKind::R_PAREN
                )
            })
        })
        .filter_map(|child| match child.kind() {
            SyntaxKind::L_CURLY | SyntaxKind::R_CURLY if options.space_after_curly_bracket => {
                insert_spaces(child)
            }
            SyntaxKind::L_SQ | SyntaxKind::R_SQ if options.space_after_square_bracket => {
                insert_spaces(child)
            }
            SyntaxKind::L_PAREN | SyntaxKind::R_PAREN if options.space_after_bracket => {
                insert_spaces(child)
            }
            SyntaxKind::L_CURLY | SyntaxKind::R_CURLY if !options.space_after_curly_bracket => {
                remove_spaces(child)
            }
            SyntaxKind::L_SQ | SyntaxKind::R_SQ if !options.space_after_square_bracket => {
                remove_spaces(child)
            }
            SyntaxKind::L_PAREN | SyntaxKind::R_PAREN if !options.space_after_bracket => {
                remove_spaces(child)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    perform_replacements(replacements);

    root
}

fn insert_spaces(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    let position = if is_opening(child.kind()) {
        if let Some(token) = child.next_sibling_or_token()?.into_token() {
            if matches!(token.kind(), SyntaxKind::WHITESPACE) {
                if token.text() == " " {
                    None
                } else {
                    Some(Position::Replace(token))
                }
            } else {
                Some(Position::After(child))
            }
        } else {
            None
        }
    } else if let Some(token) = child.prev_sibling_or_token()?.into_token() {
        if matches!(token.kind(), SyntaxKind::WHITESPACE) {
            if token.text() == " " {
                None
            } else {
                Some(Position::Replace(token))
            }
        } else {
            Some(Position::Before(child))
        }
    } else {
        unreachable!("Closing bracket should have a previous token");
    };

    position.map(|position| (position, create_token(SyntaxKind::WHITESPACE, " ")))
}

fn remove_spaces(child: SyntaxToken) -> Option<(Position, SyntaxToken)> {
    if is_opening(child.kind()) {
        if let Some(token) = child.next_sibling_or_token()?.into_token() {
            if matches!(token.kind(), SyntaxKind::WHITESPACE) {
                return Some((Position::Replace(token), create_token(SyntaxKind::ROOT, "")));
            }
        }
    } else if let Some(token) = child.prev_sibling_or_token()?.into_token() {
        if matches!(token.kind(), SyntaxKind::WHITESPACE) {
            return Some((Position::Replace(token), create_token(SyntaxKind::ROOT, "")));
        }
    }

    None
}

fn is_opening(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::L_CURLY | SyntaxKind::L_SQ | SyntaxKind::L_PAREN
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_transform() {
        crate::format_test!("invalid [sp]" => "invalid [sp]", &Default::default(), perform_pass);
        crate::format_test!("invalid {sp}" => "invalid {sp}", &Default::default(), perform_pass);
        crate::format_test!("invalid (sp)" => "invalid (sp)", &Default::default(), perform_pass);
    }

    #[test]
    fn test_transform() {
        let opts = FormatOptions {
            space_after_bracket: true,
            space_after_curly_bracket: true,
            space_after_square_bracket: true,
            ..Default::default()
        };
        crate::format_test!("invalid [sp]" => "invalid [ sp ]", &opts, perform_pass);
        crate::format_test!("invalid {sp}" => "invalid { sp }", &opts, perform_pass);
        crate::format_test!("invalid (sp)" => "invalid ( sp )", &opts, perform_pass);
    }

    #[test]
    fn test_transform_nothing_required() {
        let opts = FormatOptions {
            space_after_bracket: true,
            space_after_curly_bracket: true,
            space_after_square_bracket: true,
            ..Default::default()
        };
        crate::format_test!("invalid [ sp ]" => "invalid [ sp ]", &opts, perform_pass);
        crate::format_test!("invalid { sp }" => "invalid { sp }", &opts, perform_pass);
        crate::format_test!("invalid ( sp )" => "invalid ( sp )", &opts, perform_pass);
    }

    #[test]
    fn test_transform_early_end() {
        let opts = FormatOptions {
            space_after_bracket: true,
            space_after_curly_bracket: true,
            space_after_square_bracket: true,
            ..Default::default()
        };
        crate::format_test!("invalid [" => "invalid [", &opts, perform_pass);
        crate::format_test!("invalid {" => "invalid {", &opts, perform_pass);
        crate::format_test!("invalid (" => "invalid (", &opts, perform_pass);
    }

    #[test]
    fn test_transform_early_ins_next_line() {
        let opts = FormatOptions {
            space_after_bracket: true,
            space_after_curly_bracket: true,
            space_after_square_bracket: true,
            ..Default::default()
        };
        crate::format_test!("
invalid [
next:"
=>
"
invalid [
next:", &opts, perform_pass);

        crate::format_test!("
invalid {
next:"
=>
"
invalid {
next:", &opts, perform_pass);

        crate::format_test!("
invalid (
next:"
=>
"
invalid (
next:", &opts, perform_pass);
    }

    #[test]
    fn test_transform_remove_extra_spaces() {
        let opts = FormatOptions {
            space_after_bracket: true,
            space_after_curly_bracket: true,
            space_after_square_bracket: true,
            ..Default::default()
        };
        crate::format_test!("invalid [   sp ]" => "invalid [ sp ]", &opts, perform_pass);
        crate::format_test!("invalid { sp   }" => "invalid { sp }", &opts, perform_pass);
        crate::format_test!("invalid (  sp )" => "invalid ( sp )", &opts, perform_pass);
    }

    #[test]
    fn test_transform_remove_spaces() {
        let opts = FormatOptions {
            space_after_bracket: false,
            space_after_curly_bracket: false,
            space_after_square_bracket: false,
            ..Default::default()
        };
        crate::format_test!("invalid [ sp ]" => "invalid [sp]", &opts, perform_pass);
        crate::format_test!("invalid { sp }" => "invalid {sp}", &opts, perform_pass);
        crate::format_test!("invalid ( sp )" => "invalid (sp)", &opts, perform_pass);
    }
}
