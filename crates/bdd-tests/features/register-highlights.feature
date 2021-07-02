Feature: Register highlighting semantic tokens
  Scenario: Testing gp and fp semantic highlighting
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
    """
    str x1, [sp, #80]
    fmov d0, x1
    """
    When I run "semantic tokens" on the file "t1" at position "1:0-2:14"
    Then I expect the following response
      | delta line | delta start | length | token type  | modifiers |
      |          0 |           0 |      3 | opcode      |         0 |
      |          0 |           4 |      2 | gp-register |         0 |
      |          0 |           5 |      2 | register    |         0 |
      |          0 |           4 |      3 | number      |         0 |
      |          1 |           0 |      4 | opcode      |         0 |
      |          0 |           5 |      2 | fp-register |         0 |
      |          0 |           4 |      2 | gp-register |         0 |

