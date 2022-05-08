use std::str::FromStr;

use lsp_asm::types::DocumentPosition;

#[derive(Clone)]
pub struct PositionString {
    str: String,
}

impl PositionString {
    pub fn from_string(str: String) -> Self {
        Self { str }
    }
}

impl FromStr for PositionString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { str: s.to_string() })
    }
}

impl From<PositionString> for DocumentPosition {
    fn from(val: PositionString) -> Self {
        let mut pos = val.str.split(':');
        let line = pos.next().unwrap().parse::<u32>().unwrap() - 1;
        let column = pos.next().unwrap().parse().unwrap();

        lsp_asm::types::DocumentPosition { line, column }
    }
}

impl From<PositionString> for lsp_types::Position {
    fn from(val: PositionString) -> Self {
        let x: DocumentPosition = val.into();
        x.into()
    }
}

impl From<PositionString> for lsp_types::Range {
    fn from(pos: PositionString) -> Self {
        if pos.str.contains('-') {
            let mut range = pos.str.split('-');
            let start = PositionString::from_string(range.next().unwrap().to_string());
            let end = PositionString::from_string(range.next().unwrap().to_string());

            lsp_types::Range::new(start.into(), end.into())
        } else {
            let pos = pos.into();
            lsp_types::Range::new(pos, pos)
        }
    }
}
