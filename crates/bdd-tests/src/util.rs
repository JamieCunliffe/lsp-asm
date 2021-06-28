#![allow(deprecated)]
use lsp_asm::handler::semantic;
use lsp_asm::types::{DocumentPosition, DocumentRange};

use lsp_server::ResponseError;
use lsp_types::{
    DocumentHighlight, DocumentHighlightKind, DocumentSymbol, Location, Range, SemanticToken,
    SymbolKind, Url,
};

use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn get_doc_position(pos: &str) -> DocumentPosition {
    let mut pos = pos.split(':');
    let line = pos.next().unwrap().parse::<u32>().unwrap() - 1;
    let column = pos.next().unwrap().parse().unwrap();

    DocumentPosition { line, column }
}

pub(crate) fn make_doc_range_vec(table: &Vec<Vec<String>>) -> Vec<DocumentRange> {
    table
        .iter()
        .skip(1)
        .map(|cols| DocumentRange {
            start: get_doc_position(cols.get(0).unwrap()),
            end: get_doc_position(cols.get(1).unwrap()),
        })
        .collect::<Vec<_>>()
}

pub(crate) fn make_lsp_doc_location(file: &Url, table: &Vec<Vec<String>>) -> Vec<Location> {
    make_doc_range_vec(table)
        .drain(..)
        .map(|range| Range::new(range.start.into(), range.end.into()))
        .map(|range| Location::new(file.clone(), range))
        .collect()
}

pub(crate) fn make_doc_symbol(table: &Vec<Vec<String>>) -> Vec<DocumentSymbol> {
    let mut symbols: HashMap<u32, DocumentSymbol> = HashMap::new();

    for row in table.iter().skip(1) {
        let symbol = DocumentSymbol {
            name: row.get(1).unwrap().to_string(),
            detail: None,
            kind: match row.get(2).unwrap().as_str() {
                "function" => SymbolKind::Function,
                x => panic!("Unknown kind: {}", x),
            },
            tags: None,
            deprecated: None,
            range: make_range(row.get(3).unwrap()),
            selection_range: make_range(row.get(4).unwrap()),
            children: None,
        };
        let pid = row
            .get(5)
            .unwrap()
            .parse::<u32>()
            .unwrap_or(row.get(0).unwrap().parse().unwrap());

        if let Some(parent) = symbols.get_mut(&pid) {
            if let Some(c) = parent.children.as_mut() {
                c.push(symbol);
            } else {
                parent.children = Some(vec![symbol]);
            }
        } else {
            symbols.insert(pid, symbol);
        }
    }

    let mut v = symbols.values().cloned().collect::<Vec<_>>();
    v.sort_by(|a, b| a.range.start.partial_cmp(&b.range.start).unwrap());
    v
}

pub(crate) fn make_doc_highlight(table: &Vec<Vec<String>>) -> Vec<DocumentHighlight> {
    table
        .iter()
        .skip(1)
        .map(|row| DocumentHighlight {
            range: make_range(row.get(0).unwrap()),
            kind: match row.get(1).unwrap().as_str() {
                "text" => Some(DocumentHighlightKind::Text),
                x => panic!("Unknown kind: {}", x),
            },
        })
        .collect()
}

pub(crate) fn make_semantic(table: &Vec<Vec<String>>) -> Vec<SemanticToken> {
    table
        .iter()
        .skip(1)
        .map(|row| SemanticToken {
            delta_line: row.get(0).unwrap().parse().unwrap(),
            delta_start: row.get(1).unwrap().parse().unwrap(),
            length: row.get(2).unwrap().parse().unwrap(),
            token_type: match row.get(3).unwrap().as_str() {
                "opcode" => *semantic::OPCODE_INDEX,
                "string" => *semantic::STRING_INDEX,
                "number" => *semantic::NUMERIC_INDEX,
                "directive" => *semantic::DIRECTIVE_INDEX,
                "comment" => *semantic::COMMENT_INDEX,
                "register" => *semantic::REGISTER_INDEX,
                "metadata" => *semantic::METADATA_INDEX,
                "label" => *semantic::LABEL_INDEX,
                x => panic!("Unknown token type: {}", x),
            },
            token_modifiers_bitset: u32::from_str_radix(row.get(4).unwrap(), 2).unwrap(),
        })
        .collect()
}

pub(crate) fn make_range(range: &String) -> Range {
    let mut range = range.split('-');
    let start = get_doc_position(range.next().unwrap());
    let end = get_doc_position(range.next().unwrap());

    Range::new(start.into(), end.into())
}

pub(crate) fn make_result<T>(result: &Result<T, ResponseError>) -> Value
where
    T: serde::Serialize,
{
    match result {
        Ok(result) => serde_json::to_value(&result).unwrap(),
        Err(result) => serde_json::to_value(&result).unwrap(),
    }
}
