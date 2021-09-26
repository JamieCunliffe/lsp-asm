#![allow(deprecated)]
use base::Architecture;
use lsp_asm::config::LSPConfig;
use lsp_asm::handler::semantic;
use lsp_asm::types::{
    CompletionItem, CompletionKind, DocumentLocation, DocumentPosition, DocumentRange,
};
use lsp_server::ResponseError;
use lsp_types::{
    CodeLens, Command, CompletionList, DocumentHighlight, DocumentHighlightKind, DocumentSymbol,
    Documentation, Location, ParameterInformation, ParameterLabel, Range, SemanticToken,
    SignatureHelp, SignatureInformation, SymbolKind, Url,
};
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn parse_config(rows: &Vec<Vec<String>>) -> LSPConfig {
    let mut config: LSPConfig = Default::default();
    for row in rows.iter().skip(1) {
        let key = row[0].as_str();
        let value = row[1].as_str();
        match key {
            "architecture" => config.architecture = Architecture::from(value),
            "codelens::loc_enabled" => config.codelens.loc_enabled = value.parse::<bool>().unwrap(),
            x => panic!("Unknown configuration parameter: {}", x),
        }
    }
    config
}

pub(crate) fn get_doc_position(pos: &str) -> DocumentPosition {
    let mut pos = pos.split(':');
    let line = pos.next().unwrap().parse::<u32>().unwrap() - 1;
    let column = pos.next().unwrap().parse().unwrap();

    DocumentPosition { line, column }
}

pub(crate) fn make_doc_location_vec(table: &Vec<Vec<String>>, file: &Url) -> Vec<DocumentLocation> {
    table
        .iter()
        .skip(1)
        .map(|cols| DocumentLocation {
            uri: cols
                .get(2)
                .map(|f| Url::parse(format!("file://{}", f).as_str()).ok())
                .flatten()
                .unwrap_or_else(|| file.clone()),
            range: DocumentRange {
                start: get_doc_position(cols.get(0).unwrap()),
                end: get_doc_position(cols.get(1).unwrap()),
            },
        })
        .collect::<Vec<_>>()
}

pub(crate) fn make_lsp_doc_location(file: &Url, table: &Vec<Vec<String>>) -> Vec<Location> {
    make_doc_location_vec(table, file)
        .drain(..)
        .map(|range| range.into())
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
                "opcode" => semantic::OPCODE_INDEX,
                "string" => semantic::STRING_INDEX,
                "number" => semantic::NUMERIC_INDEX,
                "directive" => semantic::DIRECTIVE_INDEX,
                "comment" => semantic::COMMENT_INDEX,
                "register" => semantic::REGISTER_INDEX,
                "gp-register" => semantic::GP_REGISTER_INDEX,
                "fp-register" => semantic::FP_REGISTER_INDEX,
                "metadata" => semantic::METADATA_INDEX,
                "label" => semantic::LABEL_INDEX,
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

pub(crate) fn make_codelens(table: &Vec<Vec<String>>) -> Option<Vec<CodeLens>> {
    (table.len() > 1).then(|| {
        table
            .iter()
            .skip(1)
            .map(|row| CodeLens {
                range: make_range(row.get(0).unwrap()),
                command: Some(Command {
                    title: row.get(1).unwrap().into(),
                    command: row.get(2).unwrap().into(),
                    arguments: None,
                }),
                data: None,
            })
            .collect()
    })
}

pub(crate) fn make_completion(table: &Vec<Vec<String>>) -> CompletionList {
    let items = table
        .iter()
        .skip(1)
        .map(|row| CompletionItem {
            text: row.get(0).unwrap().parse().unwrap(),
            details: row.get(1).unwrap().replace("%PIPE%", "|"),
            documentation: row.get(3).map(String::from),
            kind: match row.get(2).unwrap().as_str() {
                "mnemonic" => CompletionKind::Mnemonic,
                "label" => CompletionKind::Label,
                "register" => CompletionKind::Register,
                x => panic!("Unknown completion kind: {}", x),
            },
        })
        .map(|i| i.into())
        .collect::<Vec<_>>();

    CompletionList {
        is_incomplete: true,
        items,
    }
}

pub(crate) fn sort_completions(mut list: CompletionList) -> CompletionList {
    let items = &mut list.items;
    items.sort_by(|a, b| {
        let a = (&a.label, a.detail.as_ref().unwrap());
        let b = (&b.label, b.detail.as_ref().unwrap());
        a.cmp(&b)
    });

    list
}

pub(crate) fn make_signature_help(table: &Vec<Vec<String>>) -> SignatureHelp {
    let signatures = table
        .iter()
        .skip(1)
        .map(|signature| SignatureInformation {
            label: signature.get(2).unwrap().replace("%PIPE%", "|"),
            documentation: signature
                .get(3)
                .map(|x| Documentation::String(x.to_string())),
            parameters: Some(
                signature
                    .get(4)
                    .unwrap()
                    .split("%%NEXT%%")
                    .zip(signature.get(5).unwrap().split("%%NEXT%%"))
                    .map(|(label, documentation)| ParameterInformation {
                        label: ParameterLabel::Simple(label.replace("%PIPE%", "|")),
                        documentation: Some(Documentation::String(documentation.to_string())),
                    })
                    .collect(),
            ),
            active_parameter: signature.get(1).map(|x| x.parse().unwrap()),
        })
        .collect::<Vec<_>>();

    let active = table.iter().skip(1).enumerate().find_map(|(idx, sig)| {
        sig.get(0)
            .map(|x| x == "*")
            .unwrap_or(false)
            .then(|| idx as u32)
    });

    let active_parameter = active
        .map(|a| signatures.get(a as usize).map(|s| s.active_parameter))
        .flatten()
        .flatten();

    SignatureHelp {
        signatures,
        active_signature: active,
        active_parameter,
    }
}
