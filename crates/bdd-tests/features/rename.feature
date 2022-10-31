Feature: Rename
  Scenario: Rename label
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the file "./features/test-files/file_a.s"
    When I run "rename" on the file "./features/test-files/file_a.s" at position "5:8" with the new name: filex_label
    Then I expect the following response
      | start |  end | file                           | new text     |
      |   5:6 | 5:17 | ./features/test-files/file_a.s | filex_label  |
      |   4:0 | 4:12 | ./features/test-files/file_b.s | filex_label: |
      |   1:6 | 1:17 | ./features/test-files/file_c.s | filex_label  |

    When I run "rename" on the file "./features/test-files/file_a.s" at position "4:35" with the new name: imm_new
    Then I expect the following response
      | start |  end | file                           | new text |
      |  4:31 | 4:38 | ./features/test-files/file_a.s | imm_new  |
      |   2:0 |  2:7 | ./features/test-files/file_b.s | imm_new  |

    When I run "rename" on the file "./features/test-files/file_a.s" at position "4:11" with the new name: b_register
    Then I expect the following response
      | start |  end | file                           | new text   |
      |   4:8 | 4:18 | ./features/test-files/file_a.s | b_register |
      |   1:0 | 1:10 | ./features/test-files/file_b.s | b_register |

    When I run "rename" on the file "./features/test-files/file_a.s" at position "4:21" with the new name: disallowed
    Then I expect the error "Rename on a register is unsuported"
