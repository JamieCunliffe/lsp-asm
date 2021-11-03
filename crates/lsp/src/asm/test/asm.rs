use crate::assert_listing;
use base::Architecture;

#[test]
fn test_two_instructions() {
    assert_listing!(
        r#"
pushq %rbp
popq %rbp"#,
        r#"ROOT@0..21
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..11
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
  WHITESPACE@11..12 "\n"
  INSTRUCTION@12..21
    MNEMONIC@12..16 "popq"
    WHITESPACE@16..17 " "
    REGISTER@17..21 "%rbp"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_directive() {
    pretty_env_logger::init();
    assert_listing!(
        r#".cfi_offset %rbp, -16
.p2align	4, 0x90"#,
        r#"ROOT@0..38
  DIRECTIVE@0..21
    MNEMONIC@0..11 ".cfi_offset"
    WHITESPACE@11..12 " "
    REGISTER@12..16 "%rbp"
    COMMA@16..17 ","
    WHITESPACE@17..18 " "
    NUMBER@18..21 "-16"
  WHITESPACE@21..22 "\n"
  DIRECTIVE@22..38
    MNEMONIC@22..30 ".p2align"
    WHITESPACE@30..31 "\t"
    NUMBER@31..32 "4"
    COMMA@32..33 ","
    WHITESPACE@33..34 " "
    NUMBER@34..38 "0x90"
"#,
        Architecture::X86_64
    );
}

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
fn test_parse_string() {
    assert_listing!(
        r#".ident	"Ubuntu clang version 11.0.0-++20200715091411+c86c1e972da-1~exp1~20200715072025.1806""#,
        r#"ROOT@0..92
  DIRECTIVE@0..92
    MNEMONIC@0..6 ".ident"
    WHITESPACE@6..7 "\t"
    STRING@7..92 "\"Ubuntu clang version ..."
"#
    );
}

#[test]
fn test_label() {
    assert_listing!(
        r#"main:
pushq %rbp
popq %rbp"#,
        r#"ROOT@0..26
  LABEL@0..26
    LABEL@0..5 "main:"
    WHITESPACE@5..6 "\n"
    INSTRUCTION@6..16
      MNEMONIC@6..11 "pushq"
      WHITESPACE@11..12 " "
      REGISTER@12..16 "%rbp"
    WHITESPACE@16..17 "\n"
    INSTRUCTION@17..26
      MNEMONIC@17..21 "popq"
      WHITESPACE@21..22 " "
      REGISTER@22..26 "%rbp"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_local_label() {
    {
        assert_listing!(
            r#"main:
pushq %rbp
.Lsomething:
popq %rbp"#,
            r#"ROOT@0..39
  LABEL@0..39
    LABEL@0..5 "main:"
    WHITESPACE@5..6 "\n"
    INSTRUCTION@6..16
      MNEMONIC@6..11 "pushq"
      WHITESPACE@11..12 " "
      REGISTER@12..16 "%rbp"
    WHITESPACE@16..17 "\n"
    LOCAL_LABEL@17..39
      LABEL@17..29 ".Lsomething:"
      WHITESPACE@29..30 "\n"
      INSTRUCTION@30..39
        MNEMONIC@30..34 "popq"
        WHITESPACE@34..35 " "
        REGISTER@35..39 "%rbp"
"#,
            Architecture::X86_64
        );
    }
}

#[test]
fn test_comment() {
    assert_listing!(
        r#"
pushq %rbp # This is a comment"#,
        r##"ROOT@0..31
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..31
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
    WHITESPACE@11..12 " "
    COMMENT@12..31 "# This is a comment"
"##,
        Architecture::X86_64
    );
}

#[test]
fn test_comment_like_label() {
    assert_listing!(
        r#"process:                                # @process
	.cfi_startproc
# %bb.0:"#,
        r##"ROOT@0..75
  LABEL@0..75
    LABEL@0..8 "process:"
    WHITESPACE@8..40 "                      ..."
    COMMENT@40..50 "# @process"
    WHITESPACE@50..52 "\n\t"
    DIRECTIVE@52..66
      MNEMONIC@52..66 ".cfi_startproc"
    WHITESPACE@66..67 "\n"
    COMMENT@67..75 "# %bb.0:"
"##,
        Architecture::X86_64
    );
}

#[test]
fn test_register_brackets() {
    assert_listing!(
        "movq	%rsi, -16(%rbp)",
        r#"ROOT@0..20
  INSTRUCTION@0..20
    MNEMONIC@0..4 "movq"
    WHITESPACE@4..5 "\t"
    REGISTER@5..9 "%rsi"
    COMMA@9..10 ","
    WHITESPACE@10..11 " "
    NUMBER@11..14 "-16"
    BRACKETS@14..20
      L_PAREN@14..15 "("
      REGISTER@15..19 "%rbp"
      R_PAREN@19..20 ")"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_label_arithm() {
    assert_listing!(
        ".uleb128 .Lcst_end0-.Lcst_begin0",
        r#"ROOT@0..32
  DIRECTIVE@0..32
    MNEMONIC@0..8 ".uleb128"
    WHITESPACE@8..9 " "
    TOKEN@9..19 ".Lcst_end0"
    OPERATOR@19..20 "-"
    TOKEN@20..32 ".Lcst_begin0"
"#
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
      NUMBER@19..23 "#-32"
      R_SQ@23..24 "]"
    TOKEN@24..25 "!"
"##,
        Architecture::AArch64
    );
}

#[test]
fn test_two_instructions_line_break() {
    assert_listing!(
        r#"
pushq %rbp

popq %rbp"#,
        r#"ROOT@0..22
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..11
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
  WHITESPACE@11..13 "\n\n"
  INSTRUCTION@13..22
    MNEMONIC@13..17 "popq"
    WHITESPACE@17..18 " "
    REGISTER@18..22 "%rbp"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_string_with_escaped_quote() {
    assert_listing!(
        r##".L__unnamed_709:
	.quad	.L__unnamed_1695
	.asciz	"Q\000\000\000\000\000\000\000\"\004\000\000\022\000\000"
	.size	.L__unnamed_709, 24"##,
        r#"ROOT@0..133
  LOCAL_LABEL@0..133
    LABEL@0..16 ".L__unnamed_709:"
    WHITESPACE@16..18 "\n\t"
    DIRECTIVE@18..40
      MNEMONIC@18..23 ".quad"
      WHITESPACE@23..24 "\t"
      TOKEN@24..40 ".L__unnamed_1695"
    WHITESPACE@40..42 "\n\t"
    DIRECTIVE@42..106
      MNEMONIC@42..48 ".asciz"
      WHITESPACE@48..49 "\t"
      STRING@49..106 "\"Q\\000\\000\\000\\000\\00 ..."
    WHITESPACE@106..108 "\n\t"
    DIRECTIVE@108..133
      MNEMONIC@108..113 ".size"
      WHITESPACE@113..114 "\t"
      TOKEN@114..129 ".L__unnamed_709"
      COMMA@129..130 ","
      WHITESPACE@130..131 " "
      NUMBER@131..133 "24"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_asm_with_empty_string() {
    assert_listing!(
        r#".section        .debug_loc,"",@progbits"#,
        r#"ROOT@0..39
  DIRECTIVE@0..39
    MNEMONIC@0..8 ".section"
    WHITESPACE@8..16 "        "
    TOKEN@16..26 ".debug_loc"
    COMMA@26..27 ","
    STRING@27..29 "\"\""
    COMMA@29..30 ","
    TOKEN@30..39 "@progbits"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_starts_local_label() {
    assert_listing!(
        r#"	.file	"something.c"
	.text
.Ltext0:
	.globl	main
	.type	main, @function
main:
"#,
        r#"ROOT@0..79
  WHITESPACE@0..1 "\t"
  DIRECTIVE@1..20
    MNEMONIC@1..6 ".file"
    WHITESPACE@6..7 "\t"
    STRING@7..20 "\"something.c\""
  WHITESPACE@20..22 "\n\t"
  DIRECTIVE@22..27
    MNEMONIC@22..27 ".text"
  WHITESPACE@27..28 "\n"
  LOCAL_LABEL@28..73
    LABEL@28..36 ".Ltext0:"
    WHITESPACE@36..38 "\n\t"
    DIRECTIVE@38..49
      MNEMONIC@38..44 ".globl"
      WHITESPACE@44..45 "\t"
      TOKEN@45..49 "main"
    WHITESPACE@49..51 "\n\t"
    DIRECTIVE@51..72
      MNEMONIC@51..56 ".type"
      WHITESPACE@56..57 "\t"
      TOKEN@57..61 "main"
      COMMA@61..62 ","
      WHITESPACE@62..63 " "
      TOKEN@63..72 "@function"
    WHITESPACE@72..73 "\n"
  LABEL@73..79
    LABEL@73..78 "main:"
    WHITESPACE@78..79 "\n"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_multiline_comment() {
    assert_listing!(
        r#"
pushq %rbp
/* This
is
a
comment
*/
popq %rbp"#,
        r#"ROOT@0..45
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..11
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
  WHITESPACE@11..12 "\n"
  COMMENT@12..35 "/* This\nis\na\ncomment\n*/"
  WHITESPACE@35..36 "\n"
  INSTRUCTION@36..45
    MNEMONIC@36..40 "popq"
    WHITESPACE@40..41 " "
    REGISTER@41..45 "%rbp"
"#,
        Architecture::X86_64
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

#[test]
fn test_relocation_at() {
    assert_listing!(
        "callq *_ZN4core6result19Result$LT$T$C$E$GT$2ok17h6d27845e2c2d1976E@GOTPCREL(%rip)",
        r#"ROOT@0..81
  INSTRUCTION@0..81
    MNEMONIC@0..5 "callq"
    WHITESPACE@5..6 " "
    TOKEN@6..66 "*_ZN4core6result19Res ..."
    RELOCATION@66..75 "@GOTPCREL"
    BRACKETS@75..81
      L_PAREN@75..76 "("
      REGISTER@76..80 "%rip"
      R_PAREN@80..81 ")"
"#,
        Architecture::X86_64
    );
}
