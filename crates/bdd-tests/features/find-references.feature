Feature: Testing find references
  Scenario: Find references
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "find references" on the file "./features/test-files/multiple-functions.s" at position "6:12"
    Then I expect the following response
      | start |   end |
      |   4:8 |  4:15 |
      |   6:7 |  6:14 |
      |  22:7 | 22:14 |
      | 22:28 | 22:35 |
      |  41:7 | 41:14 |
      | 78:14 | 78:21 |

  Scenario: Find references including decl
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "find references" on the file "./features/test-files/multiple-functions.s" at position "6:12" including decl
    Then I expect the following response
      | start |   end |
      |   4:8 |  4:15 |
      |   6:7 |  6:14 |
      |   7:0 |   7:8 |
      |  22:7 | 22:14 |
      | 22:28 | 22:35 |
      |  41:7 | 41:14 |
      | 78:14 | 78:21 |

  Scenario: find references for local label with multiple definitions
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/local-labels.s"
    When I run "find references" on the file "./features/test-files/local-labels.s" at position "3:3"
    Then I expect the following response
      | start |  end |
      |  2:10 | 2:15 |
      |  4:11 | 4:16 |
    When I run "find references" on the file "./features/test-files/local-labels.s" at position "7:11"
    Then I expect the following response
      | start |  end |
      |  7:10 | 7:15 |
