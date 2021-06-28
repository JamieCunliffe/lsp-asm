Feature: Testing semantic tokens
  Scenario: Request semantic tokens for a range
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "semantic tokens" on the file "./features/test-files/multiple-functions.s" at position "1:0-5:0"
    Then I expect the following response
      | delta line | delta start | length | token type | modifiers |
      |          0 |           0 |     30 | comment    |         0 |
      |          1 |           1 |      5 | directive  |         0 |
      |          1 |           1 |      5 | directive  |         0 |
      |          0 |           6 |      8 | string     |         0 |
      |          1 |           1 |      6 | directive  |         0 |
      |          0 |          31 |     27 | comment    |         0 |

