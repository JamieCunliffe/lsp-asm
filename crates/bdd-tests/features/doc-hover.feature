Feature: Document hover
  Scenario: Request document hover for number
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document hover" on the file "./features/test-files/multiple-functions.s" at position "5:14"
    Then I expect the following response
      """# Number
Decimal: 144
Hex: 0x90"""
