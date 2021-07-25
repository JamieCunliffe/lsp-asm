Feature: Debug location code lens
  Scenario: Debug location code lens
    Given an lsp initialized with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
      | codelens::loc_enabled | true    |
    When I open the temporary file "./features/test-files/debug-loc.s"
      """
	.file	2 "./features/test-files/lens.txt"
	.loc	2 1 0
	sub	sp, sp, #64
	.loc	2 6 6 is_stmt 1
      """
    When I run "codelens" on the file "./features/test-files/debug-loc.s" at position "1:0"
    Then I expect the following response
      """
[
  {
    "command": {
      "arguments": [
        {
          "range": {
            "end": {
              "character": 0,
              "line": 0
            },
            "start": {
              "character": 0,
              "line": 0
            }
          },
          "uri": "file://./features/test-files/lens.txt"
        }
      ],
      "command": "lsp-asm.loc",
      "title": "Line 0"
    },
    "range": {
      "end": {
        "character": 11,
        "line": 1
      },
      "start": {
        "character": 1,
        "line": 1
      }
    }
  },
  {
    "command": {
      "arguments": [
        {
          "range": {
            "end": {
              "character": 0,
              "line": 5
            },
            "start": {
              "character": 0,
              "line": 5
            }
          },
          "uri": "file://./features/test-files/lens.txt"
        }
      ],
      "command": "lsp-asm.loc",
      "title": "Line 5"
    },
    "range": {
      "end": {
        "character": 21,
        "line": 3
      },
      "start": {
        "character": 1,
        "line": 3
      }
    }
  }
]
      """

  Scenario: Debug location code lens
    Given an lsp initialized with the following parameters
      | key                   | value   |
      | architecture          | aarch64 |
      | codelens::loc_enabled | false   |
    When I open the temporary file "./features/test-files/debug-loc.s"
      """
	.file	2 "./features/test-files/lens.txt"
	.loc	2 1 0
	sub	sp, sp, #64
	.loc	2 6 6 is_stmt 1
      """
    When I run "codelens" on the file "./features/test-files/debug-loc.s" at position "1:0"
    Then I expect the following response
      | range | content | command |
