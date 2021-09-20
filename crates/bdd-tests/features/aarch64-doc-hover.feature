Feature: Document hover
  Scenario: Request document hover for instruction
    Given I have the "aarch64" documentation
    And an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "doc"
      """
      sub sp, sp, #48                    // =48
      stp x29, x30, [sp, #32]            // 16-byte Folded Spill
      add x29, sp, #32                   // =32
      ld1w	{ z0.s }, p0/z, [x8]
      """
    When I run "document hover" on the file "doc" at position "2:2"
    Then I expect the following response
      """
# STP V2

STP V2 Description

## Syntax:

* `STP  <Wt1>, <Wt2>, [<Xn|SP>], #<imm>`
  - **<Wt1>** Position 1
  - **<Wt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4

* `STP  <Xt1>, <Xt2>, [<Xn|SP>], #<imm>`
  - **<Xt1>** Position 1
  - **<Xt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4

* `STP  <Wt1>, <Wt2>, [<Xn|SP>, #<imm>]!`
  - **<Wt1>** Position 1
  - **<Wt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4

* `STP  <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]!`
  - **<Xt1>** Position 1
  - **<Xt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4

* `STP  <Wt1>, <Wt2>, [<Xn|SP>{, #<imm>}]`
  - **<Wt1>** Position 1
  - **<Wt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4

* `STP  <Xt1>, <Xt2>, [<Xn|SP>{, #<imm>}]`
  - **<Xt1>** Position 1
  - **<Xt2>** Position 2
  - **<Xn|SP>** Position 3
  - **<imm>** Position 4


"""
    When I run "document hover" on the file "doc" at position "4:2"
    Then I expect the following response
      """
# LD1W V1

LD1W 9

## Syntax:

* `LD1W ...`


* `LD1W ...`



"""
