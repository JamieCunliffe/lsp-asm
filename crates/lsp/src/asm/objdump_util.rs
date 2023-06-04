use syntax::ast::{find_kind_index, SyntaxKind, SyntaxNode};

/// Finds the instruction in an object dump file at the given offset to a label.
/// * node: The label node to offset into.
pub(crate) fn find_instruction_at_relative_offset(
    label: &SyntaxNode,
    offset: i128,
) -> Option<SyntaxNode> {
    assert!(matches!(label.kind(), SyntaxKind::LABEL));

    label
        .descendants()
        .filter(|node| matches!(node.kind(), SyntaxKind::INSTRUCTION))
        .find(|ins| offset_relative_to_label(label, ins) == Some(offset))
}

fn offset_relative_to_label(label: &SyntaxNode, instruction: &SyntaxNode) -> Option<i128> {
    let label_offset = i128::from_str_radix(
        find_kind_index(label, 0, SyntaxKind::OBJDUMP_OFFSET)?
            .as_token()?
            .text(),
        16,
    )
    .ok()?;
    let instruction_offset = i128::from_str_radix(
        find_kind_index(instruction, 0, SyntaxKind::OBJDUMP_OFFSET)?
            .as_token()?
            .text(),
        16,
    )
    .ok()?;

    let offset = instruction_offset - label_offset;
    Some(offset)
}
