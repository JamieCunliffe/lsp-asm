use crate::assert_listing;
use base::Architecture;

#[test]
fn test_sq_brackets() {
    assert_listing!(
        r"stp	x29, x30, [sp, -32]!",
        r#"ROOT@0..24
  INSTRUCTION@0..24
    MNEMONIC@0..3 "stp"
    WHITESPACE@3..4 "\t"
    REGISTER@4..7 "x29"
    COMMA@7..8 ","
    WHITESPACE@8..9 " "
    REGISTER@9..12 "x30"
    COMMA@12..13 ","
    WHITESPACE@13..14 " "
    BRACKETS@14..23
      L_SQ@14..15 "["
      REGISTER@15..17 "sp"
      COMMA@17..18 ","
      WHITESPACE@18..19 " "
      NUMBER@19..22 "-32"
      R_SQ@22..23 "]"
    TOKEN@23..24 "!"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_multiple() {
    assert_listing!(
        r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:
    b .L2
end:
.cfi_endproc"#,
        r#"ROOT@0..83
  LABEL@0..66
    LABEL@0..6 "entry:"
    WHITESPACE@6..7 "\n"
    DIRECTIVE@7..21
      MNEMONIC@7..21 ".cfi_startproc"
    WHITESPACE@21..26 "\n    "
    INSTRUCTION@26..50
      MNEMONIC@26..29 "stp"
      WHITESPACE@29..30 " "
      REGISTER@30..33 "x20"
      COMMA@33..34 ","
      WHITESPACE@34..35 " "
      REGISTER@35..38 "x21"
      COMMA@38..39 ","
      WHITESPACE@39..40 " "
      BRACKETS@40..49
        L_SQ@40..41 "["
        REGISTER@41..43 "sp"
        COMMA@43..44 ","
        WHITESPACE@44..45 " "
        NUMBER@45..48 "-32"
        R_SQ@48..49 "]"
      TOKEN@49..50 "!"
    WHITESPACE@50..51 "\n"
    LOCAL_LABEL@51..66
      LABEL@51..55 ".L2:"
      WHITESPACE@55..60 "\n    "
      INSTRUCTION@60..65
        MNEMONIC@60..61 "b"
        WHITESPACE@61..62 " "
        TOKEN@62..65 ".L2"
      WHITESPACE@65..66 "\n"
  LABEL@66..83
    LABEL@66..70 "end:"
    WHITESPACE@70..71 "\n"
    DIRECTIVE@71..83
      MNEMONIC@71..83 ".cfi_endproc"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_curly_brace() {
    assert_listing!(
        r#"tbl v0.8b, { v0.16b, v1.16b }, v2.8b"#,
        r#"ROOT@0..36
  INSTRUCTION@0..36
    MNEMONIC@0..3 "tbl"
    WHITESPACE@3..4 " "
    REGISTER@4..9 "v0.8b"
    COMMA@9..10 ","
    WHITESPACE@10..11 " "
    BRACKETS@11..29
      L_CURLY@11..12 "{"
      WHITESPACE@12..13 " "
      REGISTER@13..19 "v0.16b"
      COMMA@19..20 ","
      WHITESPACE@20..21 " "
      REGISTER@21..27 "v1.16b"
      WHITESPACE@27..28 " "
      R_CURLY@28..29 "}"
    COMMA@29..30 ","
    WHITESPACE@30..31 " "
    REGISTER@31..36 "v2.8b"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_negative_imm() {
    assert_listing!(
        r#"stp x29, x30, [sp, #-32]!"#,
        r##"ROOT@0..25
  INSTRUCTION@0..25
    MNEMONIC@0..3 "stp"
    WHITESPACE@3..4 " "
    REGISTER@4..7 "x29"
    COMMA@7..8 ","
    WHITESPACE@8..9 " "
    REGISTER@9..12 "x30"
    COMMA@12..13 ","
    WHITESPACE@13..14 " "
    BRACKETS@14..24
      L_SQ@14..15 "["
      REGISTER@15..17 "sp"
      COMMA@17..18 ","
      WHITESPACE@18..19 " "
      IMMEDIATE@19..20 "#"
      NUMBER@20..23 "-32"
      R_SQ@23..24 "]"
    TOKEN@24..25 "!"
"##,
        Architecture::AArch64
    );
}

#[test]
fn test_arm_comment() {
    assert_listing!(
        "ldr d0, [x9, :lo12:.LCPI0_0] // Comment",
        r#"ROOT@0..39
  INSTRUCTION@0..39
    MNEMONIC@0..3 "ldr"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "d0"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..28
      L_SQ@8..9 "["
      REGISTER@9..11 "x9"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      RELOCATION@13..19 ":lo12:"
      TOKEN@19..27 ".LCPI0_0"
      R_SQ@27..28 "]"
    WHITESPACE@28..29 " "
    COMMENT@29..39 "// Comment"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_arm_req_directive() {
    assert_listing!(
        "register .req x1",
        r#"ROOT@0..16
  ALIAS@0..16
    REGISTER_ALIAS@0..8 "register"
    WHITESPACE@8..9 " "
    MNEMONIC@9..13 ".req"
    WHITESPACE@13..14 " "
    REGISTER@14..16 "x1"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_arm_req_alias() {
    assert_listing!(
        r#"register .req x1
        mov register, x2"#,
        r#"ROOT@0..41
  ALIAS@0..16
    REGISTER_ALIAS@0..8 "register"
    WHITESPACE@8..9 " "
    MNEMONIC@9..13 ".req"
    WHITESPACE@13..14 " "
    REGISTER@14..16 "x1"
  WHITESPACE@16..25 "\n        "
  INSTRUCTION@25..41
    MNEMONIC@25..28 "mov"
    WHITESPACE@28..29 " "
    REGISTER_ALIAS@29..37 "register"
    COMMA@37..38 ","
    WHITESPACE@38..39 " "
    REGISTER@39..41 "x2"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_equ_expr() {
    assert_listing!(
        r#"two EQU 2
label_add_64 EQU label+64"#,
        r#"ROOT@0..35
  CONST_DEF@0..9
    NAME@0..3 "two"
    WHITESPACE@3..4 " "
    MNEMONIC@4..7 "EQU"
    EXPR@7..9
      WHITESPACE@7..8 " "
      NUMBER@8..9 "2"
  WHITESPACE@9..10 "\n"
  CONST_DEF@10..35
    NAME@10..22 "label_add_64"
    WHITESPACE@22..23 " "
    MNEMONIC@23..26 "EQU"
    EXPR@26..35
      WHITESPACE@26..27 " "
      TOKEN@27..32 "label"
      OPERATOR@32..33 "+"
      NUMBER@33..35 "64"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_equ_as_syntax() {
    assert_listing!(
        r#".equ number, 10
.equ label_add_64, label+64"#,
        r#"ROOT@0..43
  CONST_DEF@0..15
    MNEMONIC@0..4 ".equ"
    WHITESPACE@4..5 " "
    NAME@5..11 "number"
    COMMA@11..12 ","
    EXPR@12..15
      WHITESPACE@12..13 " "
      NUMBER@13..15 "10"
  WHITESPACE@15..16 "\n"
  CONST_DEF@16..43
    MNEMONIC@16..20 ".equ"
    WHITESPACE@20..21 " "
    NAME@21..33 "label_add_64"
    COMMA@33..34 ","
    EXPR@34..43
      WHITESPACE@34..35 " "
      TOKEN@35..40 "label"
      OPERATOR@40..41 "+"
      NUMBER@41..43 "64"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_equ_const_token() {
    assert_listing!(
        r#"two EQU 2
orr x1, x1, two"#,
        r#"ROOT@0..25
  CONST_DEF@0..9
    NAME@0..3 "two"
    WHITESPACE@3..4 " "
    MNEMONIC@4..7 "EQU"
    EXPR@7..9
      WHITESPACE@7..8 " "
      NUMBER@8..9 "2"
  WHITESPACE@9..10 "\n"
  INSTRUCTION@10..25
    MNEMONIC@10..13 "orr"
    WHITESPACE@13..14 " "
    REGISTER@14..16 "x1"
    COMMA@16..17 ","
    WHITESPACE@17..18 " "
    REGISTER@18..20 "x1"
    COMMA@20..21 ","
    WHITESPACE@21..22 " "
    CONSTANT@22..25 "two"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_arm_relocation() {
    assert_listing!(
        "ldr d0, [x9, :lo12:.LCPI0_0]",
        r#"ROOT@0..28
  INSTRUCTION@0..28
    MNEMONIC@0..3 "ldr"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "d0"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..28
      L_SQ@8..9 "["
      REGISTER@9..11 "x9"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      RELOCATION@13..19 ":lo12:"
      TOKEN@19..27 ".LCPI0_0"
      R_SQ@27..28 "]"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_arm_relocation_incomplete() {
    assert_listing!(
        "ldr d0, [x9, :lo12 ]",
        r#"ROOT@0..20
  INSTRUCTION@0..20
    MNEMONIC@0..3 "ldr"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "d0"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..20
      L_SQ@8..9 "["
      REGISTER@9..11 "x9"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      TOKEN@13..18 ":lo12"
      WHITESPACE@18..19 " "
      R_SQ@19..20 "]"
"#,
        Architecture::AArch64
    );
}
