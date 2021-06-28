Feature: Testing document highlight
  Scenario: Request highlight for label
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "21:3"
    Then I expect the following response
      |       range | kind |
      |  21:0-21:12 | text |
      | 22:16-22:27 | text |

  Scenario: Request highlight for register
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
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document highlight" on the file "./features/test-files/multiple-functions.s" at position "36:7"
    Then I expect the following response
      | range | kind |
