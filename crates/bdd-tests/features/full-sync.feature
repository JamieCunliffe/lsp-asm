Feature: Full sync file
  Scenario: Provide a full file rather than a single change
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
    When I perform a full sync of the file "t1" to bring it to version 3
      """
      str x1, [sp, #80]
      """
    When I run "syntax tree" on the file "t1" at position "1:0"
    Then I expect the following response
      """
ROOT@0..17
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
      """
