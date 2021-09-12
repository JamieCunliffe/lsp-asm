Feature: Completion of registers
  Scenario: Complete register in second position
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      stp x3, 
      """
    When I run "completion" on the file "t1" at position "1:8"
    Then I expect the following response
      | label | details | kind     |
      | x0    |         | register |
      | x1    |         | register |
      | x2    |         | register |
      | x3    |         | register |
      | x4    |         | register |
      | x5    |         | register |
      | x6    |         | register |
      | x7    |         | register |
      | x8    |         | register |
      | x9    |         | register |
      | x10   |         | register |
      | x11   |         | register |
      | x12   |         | register |
      | x13   |         | register |
      | x14   |         | register |
      | x15   |         | register |
      | x16   |         | register |
      | x17   |         | register |
      | x18   |         | register |
      | x19   |         | register |
      | x20   |         | register |
      | x21   |         | register |
      | x22   |         | register |
      | x23   |         | register |
      | x24   |         | register |
      | x25   |         | register |
      | x26   |         | register |
      | x27   |         | register |
      | x28   |         | register |
      | x29   |         | register |
      | x30   |         | register |
      | xzr   |         | register |

  Scenario: Complete register inside brackets
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      stp x29, x30, [
      """
    When I run "completion" on the file "t1" at position "1:15"
    Then I expect the following response
      | label | details | kind     |
      | sp    |         | register |
      | x0    |         | register |
      | x1    |         | register |
      | x2    |         | register |
      | x3    |         | register |
      | x4    |         | register |
      | x5    |         | register |
      | x6    |         | register |
      | x7    |         | register |
      | x8    |         | register |
      | x9    |         | register |
      | x10   |         | register |
      | x11   |         | register |
      | x12   |         | register |
      | x13   |         | register |
      | x14   |         | register |
      | x15   |         | register |
      | x16   |         | register |
      | x17   |         | register |
      | x18   |         | register |
      | x19   |         | register |
      | x20   |         | register |
      | x21   |         | register |
      | x22   |         | register |
      | x23   |         | register |
      | x24   |         | register |
      | x25   |         | register |
      | x26   |         | register |
      | x27   |         | register |
      | x28   |         | register |
      | x29   |         | register |
      | x30   |         | register |
      | xzr   |         | register |
