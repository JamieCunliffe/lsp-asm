Feature: Testing objdump offset hints
  Scenario: Offset hints in objdump file
    Given an initialized lsp
    When I open the file "./features/test-files/objdump.s"
    When I run "inlay hints" on the file "./features/test-files/objdump.s" at position "1:0-34:0"
    Then I expect the following response
      | position | label  |
      |      8:8 | (0x0)  |
      |      9:8 | (0x4)  |
      |     10:8 | (0x8)  |
      |     11:8 | (0xF)  |
      |     12:8 | (0x12) |
      |     13:8 | (0x14) |
      |     14:8 | (0x16) |
      |     15:8 | (0x1A) |
      |     20:8 | (0x0)  |
      |     21:8 | (0x4)  |
      |     22:8 | (0x6)  |
      |     23:8 | (0x9)  |
      |     24:8 | (0xA)  |
      |     25:8 | (0xD)  |
      |     26:8 | (0x11) |
      |     27:8 | (0x12) |
      |     28:8 | (0x13) |
      |     29:8 | (0x1A) |
      |     30:8 | (0x21) |
      |     31:8 | (0x28) |
      |     32:8 | (0x2E) |
      |     33:8 | (0x2F) |
