Feature: Testing documentation comments
  Scenario: Testing document comment hover
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // This is a label being defined.
      // This is the second line of documentation
      label:
          b label
      """
    When I run "document hover" on the file "t1" at position "4:8"
    Then I expect the following response
      """
      This is a label being defined.

      This is the second line of documentation
      """

  Scenario: Testing document comment hover, ignore comment on instruction
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      b label // This shouldn't be in the doc hover
      // This is a label being defined.
      // This is the second line of documentation
      label:
          b label
      """
    When I run "document hover" on the file "t1" at position "5:8"
    Then I expect the following response
      """
      This is a label being defined.

      This is the second line of documentation
      """

  Scenario: Testing document comment hover on local label
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // This is a label being defined.
      // This is the second line of documentation
      label:
          b label
      // Documentation...
      .local:
          b .local
      """
    When I run "document hover" on the file "t1" at position "7:8"
    Then I expect the following response
      """
      Documentation...
      """

  Scenario: Testing document comment hover on multiple labels
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // This is a label being defined.
      // This is the second line of documentation
      label:
          b label
      // Documentation for another...
      another:
          b another
      """
    When I run "document hover" on the file "t1" at position "7:8"
    Then I expect the following response
      """
      Documentation for another...
      """

  Scenario: Testing document comment hover on multiple labels with locals
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // This is a label being defined.
      // This is the second line of documentation
      label:
          b label
      .label_end:
      // Documentation for another...
      another:
          b another
      """
    When I run "document hover" on the file "t1" at position "8:8"
    Then I expect the following response
      """
      Documentation for another...
      """
