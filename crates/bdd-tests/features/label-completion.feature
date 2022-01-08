Feature: Completion of labels
  Scenario: handle local labels correctly in the completion of labels
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/local-labels.s"
    When I insert the following text in "./features/test-files/local-labels.s" at position "2:0" to bring it to version 3
      """
      bl 

      """
    When I run "completion" on the file "./features/test-files/local-labels.s" at position "2:3"
    Then I expect the following response
      | label | details | kind  |
      | .loop |         | label |
      | main  |         | label |
      | next  |         | label |

  Scenario: Handle documentation comments for a label
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      // This is a doc comment
      test:
          bl 
      """
    When I run "completion" on the file "t1" at position "3:7"
    Then I expect the following response
      | label | details | kind  | documentation         |
      | test  |         | label | This is a doc comment |
