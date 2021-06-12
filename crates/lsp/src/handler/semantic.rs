use std::iter;

use lsp_types::{SemanticToken, SemanticTokenType};

lazy_static! {
    pub static ref TOKEN_TYPES: Vec<SemanticTokenType> = vec![
        SemanticTokenType::KEYWORD,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::MACRO,
        SemanticTokenType::COMMENT,
        SemanticTokenType::new("register"),
        SemanticTokenType::new("label"),
        SemanticTokenType::new("metadata"),
    ];
    pub static ref OPCODE_INDEX: u32 = 0;
    pub static ref STRING_INDEX: u32 = 1;
    pub static ref NUMERIC_INDEX: u32 = 2;
    pub static ref DIRECTIVE_INDEX: u32 = 3;
    pub static ref COMMENT_INDEX: u32 = 4;
    pub static ref REGISTER_INDEX: u32 = 5;
    pub static ref LABEL_INDEX: u32 = 6;
    pub static ref METADATA_INDEX: u32 = 7;
}

pub(crate) fn semantic_delta_transform(tokens: &[SemanticToken]) -> Vec<SemanticToken> {
    let prev = tokens.iter();
    let current = tokens.iter().skip(1);

    let result = prev.zip(current).map(|(prev, current)| {
        let delta_line = current.delta_line - prev.delta_line;
        let delta_start = if delta_line != 0 {
            current.delta_start
        } else {
            current.delta_start - prev.delta_start
        };

        SemanticToken {
            delta_line,
            delta_start,
            ..*current
        }
    });

    if let Some(first) = tokens.first() {
        iter::once(*first).chain(result).collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_token(line: u32, column: u32, len: u32) -> SemanticToken {
        SemanticToken {
            delta_line: line,
            delta_start: column,
            length: len,
            token_type: 1,
            token_modifiers_bitset: 0,
        }
    }

    #[test]
    fn calculate_deltas() {
        let tokens = vec![
            make_token(0, 5, 1),
            make_token(0, 7, 1),
            make_token(1, 25, 1),
        ];
        let result = vec![
            make_token(0, 5, 1),
            make_token(0, 2, 1),
            make_token(1, 25, 1),
        ];

        assert_eq!(result, semantic_delta_transform(&tokens));
    }

    #[test]
    fn test_no_tokens() {
        assert_eq!(
            Vec::<SemanticToken>::new(),
            semantic_delta_transform(&vec![])
        );
    }
}
