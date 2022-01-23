use std::path::Path;

use lsp_types::{Location, Url};
use syntax::ast::{find_kind_index, find_parent, SyntaxKind, SyntaxToken};

use crate::handler::error::{lsp_error_map, ErrorCode};

use super::ast::LabelToken;
use super::parser::Parser;

pub(super) fn get_definition_token<'p>(
    parser: &'p Parser,
    token: &'p SyntaxToken,
) -> Result<impl Iterator<Item = SyntaxToken> + 'p, lsp_server::ResponseError> {
    let text = token.text();

    let node = if text.starts_with('.') {
        find_parent(token, SyntaxKind::LABEL)
            .ok_or_else(|| lsp_error_map(ErrorCode::MissingParentNode))?
    } else {
        parser.tree()
    };

    Ok(node
        .descendants_with_tokens()
        .filter_map(|d| d.into_token())
        .filter(|token| token.kind() == SyntaxKind::LABEL)
        .filter(move |label| {
            parser
                .token::<LabelToken>(label)
                .map(|name| name.name() == text)
                .unwrap_or(false)
        }))
}

pub(super) fn goto_definition_label(
    parser: &Parser,
    token: &SyntaxToken,
    uri: &Url,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let position = parser.position();
    Ok(get_definition_token(parser, token)?
        .filter_map(|token| {
            Some(lsp_types::Location::new(
                uri.clone(),
                position.range_for_token(&token)?.into(),
            ))
        })
        .collect())
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
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let parent = find_parent(token, SyntaxKind::DIRECTIVE)
        .or_else(|| find_parent(token, SyntaxKind::INSTRUCTION))
        .ok_or_else(|| lsp_error_map(ErrorCode::MissingParentNode))?;

    let file = find_kind_index(&parent, 0, SyntaxKind::STRING)
        .map(|t| t.into_token())
        .flatten()
        .map(|t| t.text().trim_matches('"').to_string())
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?;

    let file = Path::new(&file);
    let full_file =
        std::fs::canonicalize(file).map_err(|_| lsp_error_map(ErrorCode::FileNotFound))?;

    let uri = Url::from_file_path(full_file).map_err(|_| lsp_error_map(ErrorCode::FileNotFound))?;
    let location = lsp_types::Location::new(uri, Default::default());

    Ok(vec![location])
}

pub(crate) fn goto_definition_const(
    token: &SyntaxToken,
    parser: &Parser,
    uri: &Url,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let position = parser.position();
    let name = token.text().trim_start_matches('#');

    let def = parser
        .tree()
        .descendants()
        .filter(|d| matches!(d.kind(), SyntaxKind::CONST_DEF))
        .filter_map(|d| find_kind_index(&d, 0, SyntaxKind::NAME))
        .filter_map(|t| t.into_token())
        .filter(|t| t.text() == name)
        .filter_map(|token| {
            Some(lsp_types::Location::new(
                uri.clone(),
                position.range_for_token(&token)?.into(),
            ))
        })
        .collect();

    Ok(def)
}

pub(crate) fn goto_definition_reg_alias(
    token: &SyntaxToken,
    parser: &Parser,
    uri: &Url,
) -> Result<Vec<Location>, lsp_server::ResponseError> {
    let position = parser.position();
    let name = token.text();

    let def = parser
        .tree()
        .descendants()
        .filter(|d| matches!(d.kind(), SyntaxKind::ALIAS))
        .filter_map(|d| find_kind_index(&d, 0, SyntaxKind::REGISTER_ALIAS))
        .filter_map(|t| t.into_token())
        .filter(|t| t.text() == name)
        .filter_map(|token| {
            Some(lsp_types::Location::new(
                uri.clone(),
                position.range_for_token(&token)?.into(),
            ))
        })
        .collect();

    Ok(def)
}
