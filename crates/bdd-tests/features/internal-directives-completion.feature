Feature: Completion of internal directives
  Scenario: Complete lsp-asm-architecture
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // lsp-asm-architecture: 
      """
    When I run "completion" on the file "t1" at position "1:25"
    Then I expect the following response
      | label   | details | kind |
      | aarch64 |         | text |
      | x86-64  |         | text |
      | UNKNOWN |         | text |

  Scenario: Complete lsp-asm-architecture without docs
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | UNKNOWN |
    When I open the temporary file "t1"
      """
      lsp-asm-architecture: 
      """
    When I run "completion" on the file "t1" at position "1:22"
    Then I expect the following response
      | label   | details | kind |
      | aarch64 |         | text |
      | x86-64  |         | text |
      | UNKNOWN |         | text |
