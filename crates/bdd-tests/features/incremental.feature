Feature: Incremental updates
  Scenario: Incremental updates to add text
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      str x1, [sp, #80]
      fmov d0, x1
      """
    When I run "syntax tree" on the file "t1" at position "1:0"
    Then I expect the following response
      """
ROOT@0..29
  INSTRUCTION@0..17
    MNEMONIC@0..3 "str"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "x1"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..17
      L_SQ@8..9 "["
      REGISTER@9..11 "sp"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      NUMBER@13..16 "#80"
      R_SQ@16..17 "]"
  WHITESPACE@17..18 "\n"
  INSTRUCTION@18..29
    MNEMONIC@18..22 "fmov"
    WHITESPACE@22..23 " "
    REGISTER@23..25 "d0"
    COMMA@25..26 ","
    WHITESPACE@26..27 " "
    REGISTER@27..29 "x1"
      """
    When I insert the following text in "t1" at position "1:0" to bring it to version 3
      """
      // lsp-asm-architecture: aarch64

      """
    And I run "syntax tree" on the file "t1" at position "1:0"
    Then I expect the following response
      """
ROOT@0..62
  COMMENT@0..32 "// lsp-asm-architectu ..."
  WHITESPACE@32..33 "\n"
  INSTRUCTION@33..50
    MNEMONIC@33..36 "str"
    WHITESPACE@36..37 " "
    REGISTER@37..39 "x1"
    COMMA@39..40 ","
    WHITESPACE@40..41 " "
    BRACKETS@41..50
      L_SQ@41..42 "["
      REGISTER@42..44 "sp"
      COMMA@44..45 ","
      WHITESPACE@45..46 " "
      NUMBER@46..49 "#80"
      R_SQ@49..50 "]"
  WHITESPACE@50..51 "\n"
  INSTRUCTION@51..62
    MNEMONIC@51..55 "fmov"
    WHITESPACE@55..56 " "
    REGISTER@56..58 "d0"
    COMMA@58..59 ","
    WHITESPACE@59..60 " "
    REGISTER@60..62 "x1"
      """

  Scenario: Incremental updates to update text
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      str x1, [sp, #80]
      fmov d0, x1
      """
    When I run "syntax tree" on the file "t1" at position "1:0"
    Then I expect the following response
      """
ROOT@0..29
  INSTRUCTION@0..17
    MNEMONIC@0..3 "str"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "x1"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..17
      L_SQ@8..9 "["
      REGISTER@9..11 "sp"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      NUMBER@13..16 "#80"
      R_SQ@16..17 "]"
  WHITESPACE@17..18 "\n"
  INSTRUCTION@18..29
    MNEMONIC@18..22 "fmov"
    WHITESPACE@22..23 " "
    REGISTER@23..25 "d0"
    COMMA@25..26 ","
    WHITESPACE@26..27 " "
    REGISTER@27..29 "x1"
      """
    When I update the following text in "t1" at position "2:0-2:11" to bring it to version 3
      """

      """
    And I run "syntax tree" on the file "t1" at position "1:0"
        Then I expect the following response
      """
ROOT@0..18
  INSTRUCTION@0..17
    MNEMONIC@0..3 "str"
    WHITESPACE@3..4 " "
    REGISTER@4..6 "x1"
    COMMA@6..7 ","
    WHITESPACE@7..8 " "
    BRACKETS@8..17
      L_SQ@8..9 "["
      REGISTER@9..11 "sp"
      COMMA@11..12 ","
      WHITESPACE@12..13 " "
      NUMBER@13..16 "#80"
      R_SQ@16..17 "]"
  WHITESPACE@17..18 "\n"
      """
