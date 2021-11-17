Feature: Document hover
  Scenario: Request document hover for number
    Given an lsp initialized with the following parameters
      | key          | value  |
      | architecture | x86-64 |
    When I open the file "./features/test-files/multiple-functions.s"
    When I run "document hover" on the file "./features/test-files/multiple-functions.s" at position "5:14"
    Then I expect the following response
      """
# Number  
Decimal: 144  
Hex: 0x90
      """

  Scenario: Doc hover for label
    When I open the temporary file "t1"
      """
      __ZN7lsp_asm4main17hd69ad636b65fa5e4E:
      _ZSt16__ostream_insertIcSt11char_traitsIcEERSt13basic_ostreamIT_T0_ES6_PKS3_l:
      _ZSt4cout:
      _ZN4Test5Inner4testEv:
      """
    When I run "document hover" on the file "t1" at position "1:5"
    Then I expect the following response
      """
# Demangled Symbol  
**Rust**: `lsp_asm::main`
      """
    When I run "document hover" on the file "t1" at position "2:5"
    Then I expect the following response
      """
# Demangled Symbol  
**C++**: `std::basic_ostream<char, std::char_traits<char> >& std::__ostream_insert<char, std::char_traits<char> >(std::basic_ostream<char, std::char_traits<char> >&, char const*, long)`
      """
    When I run "document hover" on the file "t1" at position "3:5"
    Then I expect the following response
      """
# Demangled Symbol  
**C++**: `std::cout`
      """
    When I run "document hover" on the file "t1" at position "4:5"
    Then I expect the following response
      """
# Demangled Symbol  
**C++**: `Test::Inner::test()`
      """
