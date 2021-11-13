use base::Architecture;

use crate::assert_listing;

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
