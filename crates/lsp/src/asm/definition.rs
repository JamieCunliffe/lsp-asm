use std::sync::Arc;

use itertools::Itertools;
use lsp_types::{Location, Url};
use parser::parse_number;
use syntax::ast::{find_kind_index, find_parent, SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::utils::token_is_local_label;

use crate::file_util;
use crate::handler::context::Context;
use crate::handler::error::{lsp_error_map, ErrorCode};

use super::ast::LabelToken;
use super::objdump_util;
use super::parser::Parser;

pub(super) fn get_definition_token<'p, F, I>(
    context: Arc<Context>,
    parser: &'p Parser,
    token: &'p SyntaxToken,
    map: F,
) -> Result<Vec<I>, lsp_server::ResponseError>
where
    F: Fn(&Parser, &SyntaxToken) -> Option<I>,
{
    let text = token.text();
    let handle_node = |node: SyntaxNode, parser: &Parser| {
        node.descendants_with_tokens()
            .filter_map(|d| d.into_token())
            .filter(|token| token.kind() == SyntaxKind::LABEL)
            .filter(move |label| {
                parser
                    .token::<LabelToken>(label)
                    .map(|name| name.name() == text)
                    .unwrap_or(false)
            })
            .filter_map(|t| map(parser, &t))
            .collect_vec()
    };

    Ok(if token_is_local_label(token) {
        let node = find_parent(token, SyntaxKind::LABEL)
            .ok_or_else(|| lsp_error_map(ErrorCode::MissingParentNode))?;
        handle_node(node, parser)
    } else {
        context.related_parsers(true, parser.uri().clone(), |parser| {
            handle_node(parser.tree(), parser).into_iter()
        })
    })
}

pub(super) fn goto_definition_label(
    context: Arc<Context>,
    parser: &Parser,
    token: &SyntaxToken,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    match parser.file_type() {
        base::FileType::Assembly => {
            get_definition_token(context, parser, token, |parser, token| {
                Some(lsp_types::Location::new(
                    parser.uri().clone(),
                    parser.position().range_for_token(token)?.into(),
                ))
            })
        }
        base::FileType::ObjDump(_) => handle_definition_objdump(parser, token),
    }
}

pub(super) fn goto_definition_loc(
    parser: &Parser,
    token: &SyntaxToken,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    Ok(vec![parser
        .debug_map()
        .get_file_location(
            &token
                .parent()
                .ok_or_else(|| lsp_error_map(ErrorCode::MissingParentNode))?,
        )
        .map(|l| l.into())
        .ok_or_else(|| {
            lsp_error_map(ErrorCode::InvalidPosition)
        })?])
}

pub(super) fn goto_definition_label_include(
    token: &SyntaxToken,
    current: &Url,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let parent = find_parent(token, SyntaxKind::DIRECTIVE)
        .or_else(|| find_parent(token, SyntaxKind::INSTRUCTION))
        .ok_or_else(|| lsp_error_map(ErrorCode::MissingParentNode))?;

    let file = find_kind_index(&parent, 0, SyntaxKind::STRING)
        .and_then(|t| t.into_token())
        .map(|t| t.text().trim_matches('"').to_string())
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?;

    let file = file_util::make_file_relative(current.as_str(), &file)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?;

    let full_file =
        std::fs::canonicalize(file).map_err(|_| lsp_error_map(ErrorCode::FileNotFound))?;

    let uri = Url::from_file_path(full_file).map_err(|_| lsp_error_map(ErrorCode::FileNotFound))?;
    let location = lsp_types::Location::new(uri, Default::default());

    Ok(vec![location])
}

pub(crate) fn goto_definition_const(
    context: Arc<Context>,
    token: &SyntaxToken,
    parser: &Parser,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let name = token.text().trim_start_matches('#');
    let handle_node = |parser: &Parser| {
        parser
            .tree()
            .descendants()
            .filter(|d| matches!(d.kind(), SyntaxKind::CONST_DEF))
            .filter_map(|d| find_kind_index(&d, 0, SyntaxKind::NAME))
            .filter_map(|t| t.into_token())
            .filter(|t| t.text() == name)
            .filter_map(|token| {
                Some(lsp_types::Location::new(
                    parser.uri().clone(),
                    parser.position().range_for_token(&token)?.into(),
                ))
            })
            .collect_vec()
            .into_iter()
    };

    let def = context.related_parsers(true, parser.uri().clone(), handle_node);

    Ok(def)
}

pub(crate) fn goto_definition_reg_alias(
    context: Arc<Context>,
    token: &SyntaxToken,
    parser: &Parser,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let name = token.text();
    let handle_node = |parser: &Parser| {
        parser
            .tree()
            .descendants()
            .filter(|d| matches!(d.kind(), SyntaxKind::ALIAS))
            .filter_map(|d| find_kind_index(&d, 0, SyntaxKind::REGISTER_ALIAS))
            .filter_map(|t| t.into_token())
            .filter(|t| t.text() == name)
            .filter_map(|token| {
                Some(lsp_types::Location::new(
                    parser.uri().clone(),
                    parser.position().range_for_token(&token)?.into(),
                ))
            })
            .collect_vec()
            .into_iter()
    };
    let def = context.related_parsers(true, parser.uri().clone(), handle_node);

    Ok(def)
}

fn handle_definition_objdump(
    parser: &Parser,
    token: &SyntaxToken,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let text = token.text();

    let mut labels = parser
        .tree()
        .descendants_with_tokens()
        .filter_map(|d| d.into_token())
        .filter(|token| token.kind() == SyntaxKind::LABEL)
        .filter(move |label| {
            parser
                .token::<LabelToken>(label)
                .map(|name| name.name() == text)
                .unwrap_or(false)
        })
        .collect_vec();

    let mut offsets = labels
        .iter()
        .filter_map(|label| {
            let offset = parse_number(token.next_token()?.next_token()?.text()).ok()?;
            let instruction =
                objdump_util::find_instruction_at_relative_offset(&label.parent()?, offset);
            find_kind_index(&instruction?, 0, SyntaxKind::MNEMONIC)?.into_token()
        })
        .collect::<Vec<_>>();

    labels.append(&mut offsets);

    Ok(labels
        .iter()
        .filter_map(|token| {
            Some(lsp_types::Location::new(
                parser.uri().clone(),
                parser.position().range_for_token(token)?.into(),
            ))
        })
        .collect_vec())
}
