Feature: Testing objdump functions
  Scenario: Simple Goto definition
    When I open the file "./features/test-files/objdump.s"
    When I run "goto definition" on the file "./features/test-files/objdump.s" at position "45:58"
    Then I expect the following response
      | start |   end |
      | 42:17 | 42:40 |

  Scenario: Find references without declaration
    When I open the file "./features/test-files/objdump.s"
    When I run "find references" on the file "./features/test-files/objdump.s" at position "45:58"
    Then I expect the following response
      | start |   end |
      | 45:47 | 45:67 |
      | 48:47 | 48:67 |
      | 82:47 | 82:67 |

  Scenario: Find references with declaration
    When I open the file "./features/test-files/objdump.s"
    When I run "find references" on the file "./features/test-files/objdump.s" at position "45:58" including decl
    Then I expect the following response
      | start |   end |
      | 42:17 | 42:40 |
      | 45:47 | 45:67 |
      | 48:47 | 48:67 |
      | 82:47 | 82:67 |

  Scenario: Highlight registers
    When I open the file "./features/test-files/objdump.s"
    When I run "document highlight" on the file "./features/test-files/objdump.s" at position "115:42"
    Then I expect the following response
      |         range | kind |
      | 115:39-115:44 | text |
      | 115:45-115:50 | text |
      | 116:39-116:44 | text |
      | 124:53-124:58 | text |
      | 125:51-125:56 | text |
      | 126:39-126:44 | text |
      | 131:51-131:56 | text |

  Scenario: Request document hover for number (objdump)
    When I open the file "./features/test-files/objdump.s"
    When I run "document hover" on the file "./features/test-files/objdump.s" at position "102:41"
    Then I expect the following response
      """
# Number  
Decimal: 0  
Hex: 0x0
      """

  Scenario: Semantic tokens for objdump
    When I open the file "./features/test-files/objdump.s"
    When I run "semantic tokens" on the file "./features/test-files/objdump.s" at position "1:0-11:0"
    Then I expect the following response
      | delta line | delta start | length | token type | modifiers |
      |          1 |           0 |     35 | metadata   |         0 |
      |          3 |           0 |     29 | metadata   |         0 |
      |          2 |           0 |     16 | metadata   |         0 |
      |          0 |          17 |      8 | label      |         0 |
      |          1 |           2 |      6 | metadata   |         0 |
      |          0 |           6 |      1 | metadata   |         0 |
      |          0 |           2 |     21 | metadata   |         0 |
      |          0 |          22 |      7 | opcode     |         0 |
      |          1 |           2 |      6 | metadata   |         0 |
      |          0 |           6 |      1 | metadata   |         0 |
      |          0 |           2 |     21 | metadata   |         0 |
      |          0 |          22 |      3 | opcode     |         0 |
      |          0 |           7 |      4 | number     |         0 |
      |          0 |           5 |      4 | register   |         0 |
      |          1 |           2 |      6 | metadata   |         0 |
      |          0 |           6 |      1 | metadata   |         0 |
      |          0 |           2 |     21 | metadata   |         0 |
      |          0 |          22 |      3 | opcode     |         0 |
      |          0 |           7 |      6 | number     |         0 |
      |          0 |           7 |      4 | register   |         0 |
      |          0 |           6 |      4 | register   |         0 |
      |          0 |          12 |     25 | comment    |         0 |
