mod bracket_space;
mod comma_space;
mod indent;
#[cfg(test)]
mod test_util;

use base::null_as_default;
use serde::Deserialize;
use syntax::ast::SyntaxNode;

pub fn run(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode {
    ALL_PASSES
        .iter()
        .fold(root.clone_for_update(), |root, (enabled, pass)| {
            if enabled(&options.disabled_passes) {
                pass(root, options)
            } else {
                root
            }
        })
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub enum TabKind {
    #[default]
    Space,
    Tab,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FormatOptions {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub indentation_spaces: u32,
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub tab_kind: TabKind,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub space_after_bracket: bool,
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub space_after_curly_bracket: bool,
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub space_after_square_bracket: bool,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub space_after_comma: bool,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub space_before_comma: bool,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    pub disabled_passes: DisabledPasses,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indentation_spaces: 4,
            space_after_curly_bracket: false,
            space_after_bracket: false,
            space_after_square_bracket: false,
            space_before_comma: false,
            space_after_comma: true,
            tab_kind: Default::default(),
            disabled_passes: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DisabledPasses {
    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    bracket_space: bool,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    comma_space: bool,

    #[serde(deserialize_with = "null_as_default")]
    #[serde(default)]
    indent: bool,
}

macro_rules! add_pass {
    ($name:ident) => {
        (
            |enabled: &DisabledPasses| enabled.$name == false,
            $name::perform_pass,
        )
    };
}
type Formatter = fn(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode;
type EnabledFn = fn(&DisabledPasses) -> bool;

/// All the passes. Sorted in the order they should be performed.
const ALL_PASSES: &[(EnabledFn, Formatter)] = &[
    add_pass!(bracket_space),
    add_pass!(comma_space),
    add_pass!(indent),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_passes() {
        let input = "
  ADD x0, x0, x1
\tADD x0, x0, x1
DATA .EQU 0x00000001
";

        let expected = "
    ADD x0, x0, x1
    ADD x0, x0, x1
DATA .EQU 0x00000001
";

        let opts = Default::default();
        let (input, _) = crate::test_util::parse_asm(input);
        let (expected, _) = crate::test_util::parse_asm(expected);

        let actual = run(input, &opts);
        pretty_assertions::assert_eq!(format!("{expected:#?}"), format!("{actual:#?}"));
    }

    #[test]
    fn test_disabled_indent_pass() {
        let input = "
ADD x0, x0, x1
\tINVALID [r0]
";

        let expected = "
ADD x0, x0, x1
\tINVALID [ r0 ]
";

        let opts = FormatOptions {
            space_after_square_bracket: true,
            disabled_passes: DisabledPasses {
                indent: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let (input, _) = crate::test_util::parse_asm(input);
        let (expected, _) = crate::test_util::parse_asm(expected);

        let actual = run(input, &opts);
        pretty_assertions::assert_eq!(format!("{expected:#?}"), format!("{actual:#?}"));
    }
}
