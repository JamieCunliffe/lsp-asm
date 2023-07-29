Feature: codeaction insert lsp-asm-architecture directive
  Scenario: Insert lsp-asm-architecture comment
    Given an lsp initialized with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the temporary file "T1"
      """

      """
    When I run "codeaction" on the file "T1" at position "1:0"
    Then I expect the following response
      | file | name                                           | start |  end | text                                 |
      | T1   | Insert lsp-asm-architecture: aarch64 directive |   1:0 | 1:0  | // lsp-asm-architecture: aarch64{\n} |
      | T1   | Insert lsp-asm-architecture: x86-64 directive  |   1:0 | 1:0  | # lsp-asm-architecture: x86-64{\n}   |

  Scenario: lsp-asm-architecture comment already inserted
    Given an lsp initialized with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the temporary file "T1"
      """
      // lsp-asm-architecture: aarch64
      """
    When I run "codeaction" on the file "T1" at position "1:0"
    Then I expect the following response
      | file | name | start | end | text |

  Scenario: lsp-asm-architecture comment incorrect cursor position
    Given an lsp initialized with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the temporary file "T1"
      """
      label:
      
      """
    When I run "codeaction" on the file "T1" at position "2:0"
    Then I expect the following response
      | file | name | start | end | text |
