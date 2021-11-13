use crate::assert_listing;

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
fn test_label_mnemonic_line() {
    assert_listing!(
        "label: mnemonic operand1, operand2",
        r#"ROOT@0..34
  LABEL@0..34
    LABEL@0..6 "label:"
    WHITESPACE@6..7 " "
    INSTRUCTION@7..34
      MNEMONIC@7..15 "mnemonic"
      WHITESPACE@15..16 " "
      TOKEN@16..24 "operand1"
      COMMA@24..25 ","
      WHITESPACE@25..26 " "
      TOKEN@26..34 "operand2"
"#
    );
}

#[test]
fn test_label_mnemonic_line_2() {
    assert_listing!(
        r#"label: mnemonic operand1, operand2
.loc	1 3 0"#,
        r#"ROOT@0..45
  LABEL@0..45
    LABEL@0..6 "label:"
    WHITESPACE@6..7 " "
    INSTRUCTION@7..34
      MNEMONIC@7..15 "mnemonic"
      WHITESPACE@15..16 " "
      TOKEN@16..24 "operand1"
      COMMA@24..25 ","
      WHITESPACE@25..26 " "
      TOKEN@26..34 "operand2"
    WHITESPACE@34..35 "\n"
    DIRECTIVE@35..45
      MNEMONIC@35..39 ".loc"
      WHITESPACE@39..40 "\t"
      NUMBER@40..41 "1"
      WHITESPACE@41..42 " "
      NUMBER@42..43 "3"
      WHITESPACE@43..44 " "
      NUMBER@44..45 "0"
"#
    );
}
