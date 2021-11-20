Feature: Diagnostics with assembler_flags
  Scenario: Request diagnosics for a file with assembler_flags
    Given an lsp initialized in "./features/flags/" with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the file "./features/flags/test.s"
    When I run diagnostics on the file "./features/flags/test.s"
    Then I expect the following errors
      | file                    | line | column | level | description       |
      | ./features/flags/test.s |    1 |      5 | error | unknown directive |
  Scenario: Only send diagnosics for the file requested
    Given an lsp initialized in "./features/flags/" with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
    When I open the file "./features/flags/main.s"
    When I run diagnostics on the file "./features/flags/main.s"
    Then I expect the following errors
      | file                    | line | column | level | description       |
