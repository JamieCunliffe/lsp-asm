Feature: Testing document highlight
  Scenario: Request highlight for label
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "21:3"
    Then I expect the following response
      |       range | kind |
      |  21:0-21:12 | text |
      | 22:16-22:27 | text |
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "28:3"
    Then I expect the following response
      |       range | kind |
      |  25:8-25:27 | text |
      |  27:7-27:26 | text |
      |  28:0-28:20 | text |
      |  47:7-47:26 | text |
      | 47:40-47:59 | text |
      |  66:7-66:26 | text |
      | 79:14-79:33 | text |

  Scenario: Request highlight for local label with multiple definitions
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/local-labels.s"
    When I run "document highlight" on the file "./features/test-files/local-labels.s" at position "3:3"
    Then I expect the following response
      |     range | kind |
      | 2:10-2:15 | text |
      |   3:0-3:6 | text |
      | 4:11-4:16 | text |
    When I run "document highlight" on the file "./features/test-files/local-labels.s" at position "7:11"
    Then I expect the following response
      |     range | kind |
      |   6:0-6:6 | text |
      | 7:10-7:15 | text |

  Scenario: Request highlight for register
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "16:18"
    Then I expect the following response
      |       range | kind |
      | 16:16-16:20 | text |
      | 17:10-17:14 | text |
      | 38:16-38:20 | text |
      | 39:11-39:15 | text |
      |  40:6-40:10 | text |

  Scenario: Request highlight for number
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "36:7"
    Then I expect the following response
      | range | kind |

  Scenario: Request highlight for register on last line
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the temporary file "t1"
      """
        movl %edi, -4(%rbp)
        movl -4(%rbp), %eax
      """
    When I run "document highlight" on the file "t1" at position "1:16"
    Then I expect the following response
      |     range | kind |
      | 1:14-1:18 | text |
      | 2:8-2:12  | text |
