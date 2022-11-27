#[cfg(test)]
mod test_util;

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
#[serde(default)]
pub struct FormatOptions {
    pub indentation_spaces: u32,
    pub tab_kind: TabKind,

    pub space_after_bracket: bool,
    pub space_after_curly_bracket: bool,
    pub space_after_square_bracket: bool,
    pub space_after_comma: bool,
    pub space_before_comma: bool,
    pub space_around_operators: bool,
    pub newline_after_label: bool,

    pub align_first_operand: bool,

    pub align_register_alias: bool,
    pub align_const_defs: bool,

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
            space_around_operators: true,
            newline_after_label: true,
            align_first_operand: false,
            align_register_alias: true,
            align_const_defs: true,
            tab_kind: Default::default(),
            disabled_passes: Default::default(),
        }
    }
}

impl FormatOptions {
    pub fn make_indentation(&self) -> String {
        match self.tab_kind {
            TabKind::Space => " ".repeat(self.indentation_spaces as usize),
            TabKind::Tab => String::from("\t"),
        }
    }
}

macro_rules! define_passes {
    ($($name:ident),*) => {
        $(
            mod $name;
        )*

        #[derive(Clone, Debug, Default, Deserialize)]
        #[serde(default)]
        pub struct DisabledPasses {
            $(
                $name: bool
            ),*
        }

        const ALL_PASSES: &[(EnabledFn, Formatter)] = &[
            $(
                (
                    |enabled: &DisabledPasses| enabled.$name == false,
                    $name::perform_pass,
                )
            ),*
        ];
    };
}

type Formatter = fn(root: SyntaxNode, options: &FormatOptions) -> SyntaxNode;
type EnabledFn = fn(&DisabledPasses) -> bool;

// All the passes. Sorted in the order they should be performed.
define_passes!(
    bracket_space,
    align_first_operand,
    comma_space,
    space_around_operators,
    label_newline,
    const_def_align,
    indent
);

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
