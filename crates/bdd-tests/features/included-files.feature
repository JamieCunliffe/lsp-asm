Feature: Multiple files
  Scenario: Request hover for alias defined in another file
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | aarch64 |
    When I open the file "./features/test-files/file_a.s"
    When I run "document hover" on the file "./features/test-files/file_a.s" at position "4:10"
    Then I expect the following response
      """
      `a_register` is an alias to register `x20`
      """
    When I run "find references" on the file "./features/test-files/file_a.s" at position "4:34"
    Then I expect the following response
      | start |  end | file                           |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |

    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "5:9"
    Then I expect the following response
      | start |  end | file                           |
      |   4:0 | 4:12 | ./features/test-files/file_b.s |

    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "4:34"
    Then I expect the following response
      | start |  end | file                           |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |

    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "4:11"
    Then I expect the following response
      | start |  end | file                           |
      |   1:0 | 1:10 | ./features/test-files/file_b.s |
