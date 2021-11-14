use crate::assert_listing;
use base::Architecture;

#[test]
fn test_multiline_comment_not_closed() {
    assert_listing!(
        r#"
pushq %rbp
/* This
is
a
comment
"#,
        r#"ROOT@0..33
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..11
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
  WHITESPACE@11..12 "\n"
  COMMENT@12..33 "/* This\nis\na\ncomment\n"
"#,
        Architecture::X86_64
    );
}

#[test]
fn test_unclosed_bracket() {
    assert_listing!(
        "str w8, [sp",
        r#"ROOT@0..11
  INSTRUCTION@0..11
    MNEMONIC@0..3 "str"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "w8"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..11
      L_SQ@8..9 "["
      REGISTER@9..11 "sp"
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_unclosed_bracket_no_text() {
    assert_listing!(
        "str w8, [",
        r#"ROOT@0..9
  INSTRUCTION@0..9
    MNEMONIC@0..3 "str"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "w8"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..9
      L_SQ@8..9 "["
"#,
        Architecture::AArch64
    );
}

#[test]
fn test_incomplete_multiline_comment_ins_line() {
    assert_listing!(
        r#"
pushq %rbp /* This
is
a
comment
"#,
        r#"ROOT@0..33
  WHITESPACE@0..1 "\n"
  INSTRUCTION@1..12
    MNEMONIC@1..6 "pushq"
    WHITESPACE@6..7 " "
    REGISTER@7..11 "%rbp"
    WHITESPACE@11..12 " "
  COMMENT@12..33 "/* This\nis\na\ncomment\n"
"#,
        Architecture::X86_64
    );
}
