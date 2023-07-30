use lsp_types::{InlayHint, InlayHintLabel};
use rowan::TextRange;
use syntax::ast::{find_parent, SyntaxKind};

use crate::asm::objdump_util::offset_relative_to_label;

use super::parser::Parser;

pub(super) fn objdump_inlay_hints(parser: &Parser, location: TextRange) -> Vec<InlayHint> {
    let position = parser.position();
    parser
        .tokens_in_range(location)
        .filter(|token| matches!(token.kind(), SyntaxKind::OBJDUMP_OFFSET))
        .filter_map(|token| {
            let offset = offset_relative_to_label(
                &find_parent(&token, SyntaxKind::LABEL)?,
                &find_parent(&token, SyntaxKind::INSTRUCTION)?,
            )?;

            Some(InlayHint {
                position: position.get_end_position(&token)?.into(),
                label: InlayHintLabel::String(format!("({offset:#X})")),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: None,
                padding_right: None,
                data: None,
            })
        })
        .collect()
}
