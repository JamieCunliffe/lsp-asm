Feature: Multiple files
  Scenario: Request hover for alias defined in another file
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/file_a.s"
    When I run "document hover" on the file "./features/test-files/file_a.s" at position "4:10"
    Then I expect the following response
      """
      `a_register` is an alias to register `x20`
      """

    # Immediate
    When I run "find references" on the file "./features/test-files/file_a.s" at position "4:34" including decl
    Then I expect the following response
      | start |  end | file                           |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |

    When I run "find references" on the file "./features/test-files/file_b.s" at position "2:2" including decl
    Then I expect the following response
      | start |  end | file                           |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |

    When I run "find references" on the file "./features/test-files/file_a.s" at position "4:34"
    Then I expect the following response
      | start |  end | file                           |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |

    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "4:34"
    Then I expect the following response
      | start | end | file                           |
      |   2:0 | 2:7 | ./features/test-files/file_b.s |

    # Register Alias
    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "4:11"
    Then I expect the following response
      | start |  end | file                           |
      |   1:0 | 1:10 | ./features/test-files/file_b.s |

    # Label
    When I run "goto definition" on the file "./features/test-files/file_a.s" at position "5:9"
    Then I expect the following response
      | start |  end | file                           |
      |   4:0 | 4:12 | ./features/test-files/file_b.s |

    When I run "find references" on the file "./features/test-files/file_a.s" at position "5:8" including decl
    Then I expect the following response
      | start |  end | file                           |
      |   5:6 | 5:17 | ./features/test-files/file_a.s |
      |   4:0 | 4:12 | ./features/test-files/file_b.s |
      |   1:6 | 1:17 | ./features/test-files/file_c.s |

    # Local label
    When I run "find references" on the file "./features/test-files/file_a.s" at position "6:8" including decl
    Then I expect the following response
      | start |  end | file                           |
      |   6:6 | 6:17 | ./features/test-files/file_a.s |
      |   7:0 | 7:12 | ./features/test-files/file_a.s |

  Scenario: Closing a file doesn't break included files
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/file_a.s"
    When I open the file "./features/test-files/file_b.s"
    When I run "find references" on the file "./features/test-files/file_a.s" at position "4:34" including decl
    Then I expect the following response
      | start |  end | file                           |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |
    When I close the file "./features/test-files/file_b.s"
    When I run "find references" on the file "./features/test-files/file_a.s" at position "4:34" including decl
    Then I expect the following response
      | start |  end | file                           |
      |  4:31 | 4:38 | ./features/test-files/file_a.s |
      |   2:0 |  2:7 | ./features/test-files/file_b.s |
