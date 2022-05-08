Feature: Diagnostics with assembler_flags
  Scenario: Request diagnosics for a file with assembler_flags
    Given an lsp initialized in "./features/flags/" with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the file "./features/flags/test.s"
    Then I expect the following errors for "./features/flags/test.s"
      | line | column | level | description       |
      |    1 |      5 | error | unknown directive |

  Scenario: Only send diagnosics for the file requested
    Given an lsp initialized in "./features/flags/" with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the file "./features/flags/main.s"
    Then I expect the following errors for "./features/flags/main.s"
      | line | column | level | description |
