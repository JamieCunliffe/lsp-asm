use arch::register_names::AARCH64_REGISTERS;
use base::FileType;
use parser::config::ParserConfig;
use parser::{ParsedData, ParsedInclude};
use syntax::alias::Alias;
use syntax::ast::SyntaxNode;

use crate::Instruction;

pub(super) fn parse_asm(data: &str, file_type: FileType) -> (SyntaxNode, Alias) {
    let config = ParserConfig {
        comment_start: String::from("//"),
        architecture: base::Architecture::AArch64,
        file_type,
        registers: Some(&AARCH64_REGISTERS),
    };
    let load_file = |_current_config: &ParserConfig,
                     _current_file: &str,
                     _filename: &str|
     -> Option<ParsedInclude> { None };

    let ParsedData { root, alias, .. } = parser::parse_asm(data, &config, None, load_file);
    (SyntaxNode::new_root(root), alias)
}

pub(super) fn make_instruction() -> Instruction {
    let data = r#"
{
  "opcode": "stp",
  "header": "stp",
  "architecture": null,
  "description": "The stp instruction",
  "asm_template": [
    {
      "asm": [
        "stp  <gp_32>, <gp_32>, [<gp|sp_64>], #<imm>"
      ],
      "display_asm": "stp  <Wt1>, <Wt2>, [<Xn|SP>], #<imm>",
      "items": [
        {
          "name": "<Wt1>",
          "description": "Position 0"
        },
        {
          "name": "<Wt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    },
    {
      "asm": [
        "stp  <gp_64>, <gp_64>, [<gp|sp_64>], #<imm>"
      ],
      "display_asm": "stp  <Xt1>, <Xt2>, [<Xn|SP>], #<imm>",
      "items": [
        {
          "name": "<Xt1>",
          "description": "Position 0"
        },
        {
          "name": "<Xt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    },
    {
      "asm": [
        "stp  <gp_32>, <gp_32>, [<gp|sp_64>, #<imm>]!"
      ],
      "display_asm": "stp  <Wt1>, <Wt2>, [<Xn|SP>, #<imm>]!",
      "items": [
        {
          "name": "<Wt1>",
          "description": "Position 0"
        },
        {
          "name": "<Wt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    },
    {
      "asm": [
        "stp  <gp_64>, <gp_64>, [<gp|sp_64>, #<imm>]!"
      ],
      "display_asm": "stp  <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!",
      "items": [
        {
          "name": "<Xt1>",
          "description": "Position 0"
        },
        {
          "name": "<Xt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    },
    {
      "asm": [
        "stp  <gp_32>, <gp_32>, [<gp|sp_64>, #<imm>]",
        "stp  <gp_32>, <gp_32>, [<gp|sp_64>]"
      ],
      "display_asm": "stp  <Wt1>, <Wt2>, [<Xn|SP>{, #<imm>}]",
      "items": [
        {
          "name": "<Wt1>",
          "description": "Position 0"
        },
        {
          "name": "<Wt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    },
    {
      "asm": [
        "stp  <gp_64>, <gp_64>, [<gp|sp_64>, #<imm>]",
        "stp  <gp_64>, <gp_64>, [<gp|sp_64>]"
      ],
      "display_asm": "stp  <Xt1>, <Xt2>, [<Xn|SP>{, #<imm>}]",
      "items": [
        {
          "name": "<Xt1>",
          "description": "Position 0"
        },
        {
          "name": "<Xt2>",
          "description": "Position 1"
        },
        {
          "name": "<Xn|SP>",
          "description": "Position 2"
        },
        {
          "name": "<imm>",
          "description": "Position 3"
        }
      ]
    }
  ]
}
"#;
    serde_json::from_str(data).unwrap()
}

pub(super) fn make_label_instruction() -> Instruction {
    let data = r#"{
      "opcode": "BL",
      "header": "BL",
      "architecture": null,
      "description": "bl",
      "asm_template": [
        {
          "asm": [
            "BL  <label>"
          ],
          "display_asm": "BL <label>",
          "items": [
            {
              "name": "<label>",
              "description": "label"
            }
          ]
        }
      ]
    }"#;
    serde_json::from_str(data).unwrap()
}
