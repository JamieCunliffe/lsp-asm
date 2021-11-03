Feature: Testing equ directive
  Scenario: infix equ directive
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      number EQU 12
      other EQU 16
      str wzr, [sp, number]
      str wzr, [sp, #number]
      """
    When I run "document hover" on the file "t1" at position "3:16"
    Then I expect the following response
      """
      `number` is defined as `12`
      """
    When I run "document highlight" on the file "t1" at position "3:16"
    Then I expect the following response
      |     range | kind |
      |  1:0-1:6  | text |
      | 3:14-3:20 | text |
      | 4:14-4:21 | text |
    When I run "goto definition" on the file "t1" at position "3:16"
    Then I expect the following response
      | start |  end |
      |   1:0 | 1:6 |
    When I run "goto definition" on the file "t1" at position "4:16"
    Then I expect the following response
      | start | end |
      |   1:0 | 1:6 |
    When I run "find references" on the file "t1" at position "3:16"
    Then I expect the following response
      | start |  end |
      |  3:14 | 3:20 |
      |  4:14 | 4:21 |
    When I run "find references" on the file "t1" at position "4:16" including decl
    Then I expect the following response
      | start |  end |
      |   1:0 |  1:6 |
      |  3:14 | 3:20 |
      |  4:14 | 4:21 |

  Scenario: prefix equ directive
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      .equ number, 12
      .equ other, 16
      str wzr, [sp, number]
      str wzr, [sp, #number]
      """
    When I run "document hover" on the file "t1" at position "3:16"
    Then I expect the following response
      """
      `number` is defined as `12`
      """
    When I run "document highlight" on the file "t1" at position "3:16"
    Then I expect the following response
      |     range | kind |
      |  1:5-1:11 | text |
      | 3:14-3:20 | text |
      | 4:14-4:21 | text |
    When I run "goto definition" on the file "t1" at position "3:16"
    Then I expect the following response
      | start |  end |
      |   1:5 | 1:11 |
    When I run "goto definition" on the file "t1" at position "4:16"
    Then I expect the following response
      | start |  end |
      |   1:5 | 1:11 |
    When I run "find references" on the file "t1" at position "3:16"
    Then I expect the following response
      | start |  end |
      |  3:14 | 3:20 |
      |  4:14 | 4:21 |
    When I run "find references" on the file "t1" at position "4:16" including decl
    Then I expect the following response
      | start |  end |
      |   1:5 | 1:11 |
      |  3:14 | 3:20 |
      |  4:14 | 4:21 |
