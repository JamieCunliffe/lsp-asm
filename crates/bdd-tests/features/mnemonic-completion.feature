Feature: Completion of mnemonic
  Scenario: Complete mnemonic with nothing on line
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      main:

      """
    When I run "completion" on the file "t1" at position "2:0"
    # LD1W is deduplicated here due to the display asm being the same for each in our test json.
    Then I expect the following response
      | label | details                                     | kind     | documentation      |
      | BL    | BL <label>                                  | mnemonic | bl                 |
      | LD1W  | LD1W ...                                    | mnemonic | LD1W 0             |
      | LD1W  | LD1W ...                                    | mnemonic | LD1W 18            |
      | LD1W  | LD1W ...                                    | mnemonic | LD1W 27            |
      | LD1W  | LD1W ...                                    | mnemonic | LD1W 9             |
      | STP   | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>, #<imm>]!  | mnemonic | STP V1 Description |
      | STP   | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>], #<imm>   | mnemonic | STP V1 Description |
      | STP   | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>{, #<imm>}] | mnemonic | STP V1 Description |
      | STP   | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>, #<imm>]!  | mnemonic | STP V1 Description |
      | STP   | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>], #<imm>   | mnemonic | STP V1 Description |
      | STP   | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>{, #<imm>}] | mnemonic | STP V1 Description |
      | STP   | STP  <St1>, <St2>, [<Xn%PIPE%SP>, #<imm>]!  | mnemonic | STP V1 Description |
      | STP   | STP  <St1>, <St2>, [<Xn%PIPE%SP>], #<imm>   | mnemonic | STP V1 Description |
      | STP   | STP  <St1>, <St2>, [<Xn%PIPE%SP>{, #<imm>}] | mnemonic | STP V1 Description |
      | STP   | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>, #<imm>]!  | mnemonic | STP V2 Description |
      | STP   | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>], #<imm>   | mnemonic | STP V2 Description |
      | STP   | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>{, #<imm>}] | mnemonic | STP V2 Description |
      | STP   | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>, #<imm>]!  | mnemonic | STP V2 Description |
      | STP   | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>], #<imm>   | mnemonic | STP V2 Description |
      | STP   | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>{, #<imm>}] | mnemonic | STP V2 Description |
