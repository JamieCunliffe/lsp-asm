Feature: Testing goto definition
  Scenario: Simple Goto definition
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "4:9"
    Then I expect the following response
      | start | end |
      |   7:0 | 7:8 |

  Scenario: Goto definition no token
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "11:0"
    Then I expect the following response
      | start | end |

  Scenario: Goto definition on number
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "11:21"
    Then I expect the following response
      | start | end |

  Scenario: Goto definition no token
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "goto definition" on the file "./features/test-files/multiple-functions.s" at position "15:3"
    Then I expect the following response
      | start | end |

  Scenario: Goto definition on .loc directive
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the temporary file "t1"
      """
Lfunc_begin0:
	.file	2 "./features/test-files/debugloc.txt"
	.loc	2 1 0
	.cfi_startproc
      """
    When I run "goto definition" on the file "t1" at position "3:3"
    Then I expect the following response
      | start | end | file                               |
      |   1:0 | 1:0 | ./features/test-files/debugloc.txt |

  Scenario: Goto definition on .include
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/file_a.s"
    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "1:3"
    Then I expect the following response
      | start | end | file                           |
      |   1:0 | 1:0 | ./features/test-files/file_b.s |
    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "1:13"
    Then I expect the following response
      | start | end | file                             |
      |   1:0 | 1:0 | ./features/test-files/file_b.s   |

  Scenario: Goto definition local label
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/local-labels.s"
    When I run "goto definition" on the file "./features/test-files/local-labels.s" at position "7:12"
    Then I expect the following response
      | start | end |
      |   6:0 | 6:6 |
    When I run "goto definition" on the file "./features/test-files/local-labels.s" at position "4:14"
    Then I expect the following response
      | start | end |
      |   3:0 | 3:6 |

  Scenario: Test goto definition for a register alias
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
    """
    register .req x1
    str register, [sp, #80]
    """
    When I run "goto definition" on the file "t1" at position "2:6"
    Then I expect the following response
      | start | end | file |
      |   1:0 | 1:8 | t1   |
