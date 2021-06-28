Feature: Testing document symbols
  Scenario: Request document symbols
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document symbols" on the file "./features/test-files/multiple-functions.s" at position "1:0"
    Then I expect the following response
      | id | name                 | kind     |     range | sel_range | p_id |
      |  1 | process:             | function |  7:0-28:0 |  7:0-28:0 |      |
      |  2 | .Lfunc_end0:         | function | 21:0-28:0 | 21:0-28:0 |    1 |
      |  3 | some_other_function: | function | 28:0-53:0 | 28:0-53:0 |      |
      |  4 | .Lfunc_end1:         | function | 46:0-53:0 | 46:0-53:0 |    3 |
      |  5 | main:                | function | 53:0-80:0 | 53:0-80:0 |      |
      |  6 | .Lfunc_end2:         | function | 71:0-80:0 | 71:0-80:0 |    5 |
