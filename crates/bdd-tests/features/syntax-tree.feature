Feature: View syntax tree
  Scenario: View syntax tree
    Given an lsp initialized with the following parameters
      | key                   | value  |
      | architecture          | x86-64 |
      | codelens::loc_enabled | true   |
    When I open the temporary file "t1"
      """
      subq $56, %rsp
      """
    When I run "syntax tree" on the file "t1" at position "1:0"
    Then I expect the following response
      """ROOT@0..14
  INSTRUCTION@0..14
    MNEMONIC@0..4 "subq"
    WHITESPACE@4..5 " "
    NUMBER@5..8 "$56"
    COMMA@8..9 ","
    WHITESPACE@9..10 " "
    REGISTER@10..14 "%rsp"
      """
