use crate::documentation::Instruction;

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
        "stp  <Wt1>, <Wt2>, [<Xn|SP>], #<imm>"
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
        "stp  <Xt1>, <Xt2>, [<Xn|SP>], #<imm>"
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
        "stp  <Wt1>, <Wt2>, [<Xn|SP>, #<imm>]!"
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
        "stp  <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!"
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
        "stp  <Wt1>, <Wt2>, [<Xn|SP>, #<imm>]",
        "stp  <Wt1>, <Wt2>, [<Xn|SP>]"
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
        "stp  <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]",
        "stp  <Xt1>, <Xt2>, [<Xn|SP>]"
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
