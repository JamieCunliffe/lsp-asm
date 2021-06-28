Feature: Testing goto definition
  Scenario: Simple Goto definition
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "4:9"
    Then I expect the following response
      | start | end |
      |   7:0 | 7:8 |

  Scenario: Goto definition no token
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "11:0"
    Then I expect the following response
      | start | end |

  Scenario: Goto definition on number
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "11:21"
    Then I expect the following response
      | start | end |

  Scenario: Goto definition no token
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "15:3"
    Then I expect the following response
      | start | end |
