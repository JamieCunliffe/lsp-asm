Feature: Display of signature
  Scenario: Show signature for multiple variants
    Given I have the "aarch64" documentation
    Given an lsp initialized with the following parameters
      | key          | value   |
      | architecture | aarch64 |
    When I open the temporary file "t1"
      """
      STP 
      """
    When I run "signature help" on the file "t1" at position "1:4"
    Then I expect the following response
      | active | active parameter | label                                       | documentation      | parameter label                                     | parameter documentation                                          |
      | *      |                0 | STP  <St1>, <St2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <St1>, <St2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <St1>, <St2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                0 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |

    When I insert the following text in "t1" at position "1:4" to bring it to version 2
      """
      q0, 
      """
    When I run "signature help" on the file "t1" at position "1:8"
    Then I expect the following response
      | active | active parameter | label                                       | documentation      | parameter label                                     | parameter documentation                                          |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      | *      |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
    When I update the following text in "t1" at position "1:4-1:6" to bring it to version 3
      """
      w0
      """
    When I run "signature help" on the file "t1" at position "1:8"
    Then I expect the following response
      | active | active parameter | label                                       | documentation      | parameter label                                     | parameter documentation                                          |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>], #<imm>   | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <St1>, <St2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <St1>%%NEXT%%<St2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Dt1>, <Dt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Dt1>%%NEXT%%<Dt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Qt1>, <Qt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V1 Description | <Qt1>%%NEXT%%<Qt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      | *      |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>], #<imm>   | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>, #<imm>]!  | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Wt1>, <Wt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Wt1>%%NEXT%%<Wt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
      |        |                1 | STP  <Xt1>, <Xt2>, [<Xn%PIPE%SP>{, #<imm>}] | STP V2 Description | <Xt1>%%NEXT%%<Xt2>%%NEXT%%<Xn%PIPE%SP>%%NEXT%%<imm> | Position 1%%NEXT%%Position 2%%NEXT%%Position 3%%NEXT%%Position 4 |
