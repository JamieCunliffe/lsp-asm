use crate::assert_listing;

#[test]
fn test_parse_objdump() {
    assert_listing!(
        r#"
a.out:     file format elf64-x86-64


Disassembly of section .text:

00000000002015a0 <_start>:
  2015a0:	f3 0f 1e fa          	endbr64
  2015a4:	31 ed                	xor    %ebp,%ebp
  2015a6:	49 89 d1             	mov    %rdx,%r9
  2015a9:	5e                   	pop    %rsi
  2015aa:	48 89 e2             	mov    %rsp,%rdx
  2015ad:	48 83 e4 f0          	and    $0xfffffffffffffff0,%rsp
"#,
        r#"ROOT@0..391
  WHITESPACE@0..1 "\n"
  METADATA@1..36 "a.out:     file forma ..."
  WHITESPACE@36..39 "\n\n\n"
  METADATA@39..68 "Disassembly of sectio ..."
  WHITESPACE@68..70 "\n\n"
  LABEL@70..391
    METADATA@70..86 "00000000002015a0"
    WHITESPACE@86..87 " "
    LABEL@87..96 "<_start>:"
    WHITESPACE@96..99 "\n  "
    INSTRUCTION@99..136
      METADATA@99..105 "2015a0"
      METADATA@105..106 ":"
      WHITESPACE@106..107 "\t"
      METADATA@107..128 "f3 0f 1e fa          "
      WHITESPACE@128..129 "\t"
      MNEMONIC@129..136 "endbr64"
    WHITESPACE@136..139 "\n  "
    INSTRUCTION@139..185
      METADATA@139..145 "2015a4"
      METADATA@145..146 ":"
      WHITESPACE@146..147 "\t"
      METADATA@147..168 "31 ed                "
      WHITESPACE@168..169 "\t"
      MNEMONIC@169..172 "xor"
      WHITESPACE@172..176 "    "
      TOKEN@176..180 "%ebp"
      COMMA@180..181 ","
      TOKEN@181..185 "%ebp"
    WHITESPACE@185..188 "\n  "
    INSTRUCTION@188..233
      METADATA@188..194 "2015a6"
      METADATA@194..195 ":"
      WHITESPACE@195..196 "\t"
      METADATA@196..217 "49 89 d1             "
      WHITESPACE@217..218 "\t"
      MNEMONIC@218..221 "mov"
      WHITESPACE@221..225 "    "
      REGISTER@225..229 "%rdx"
      COMMA@229..230 ","
      TOKEN@230..233 "%r9"
    WHITESPACE@233..236 "\n  "
    INSTRUCTION@236..277
      METADATA@236..242 "2015a9"
      METADATA@242..243 ":"
      WHITESPACE@243..244 "\t"
      METADATA@244..265 "5e                   "
      WHITESPACE@265..266 "\t"
      MNEMONIC@266..269 "pop"
      WHITESPACE@269..273 "    "
      REGISTER@273..277 "%rsi"
    WHITESPACE@277..280 "\n  "
    INSTRUCTION@280..326
      METADATA@280..286 "2015aa"
      METADATA@286..287 ":"
      WHITESPACE@287..288 "\t"
      METADATA@288..309 "48 89 e2             "
      WHITESPACE@309..310 "\t"
      MNEMONIC@310..313 "mov"
      WHITESPACE@313..317 "    "
      REGISTER@317..321 "%rsp"
      COMMA@321..322 ","
      REGISTER@322..326 "%rdx"
    WHITESPACE@326..329 "\n  "
    INSTRUCTION@329..390
      METADATA@329..335 "2015ad"
      METADATA@335..336 ":"
      WHITESPACE@336..337 "\t"
      METADATA@337..358 "48 83 e4 f0          "
      WHITESPACE@358..359 "\t"
      MNEMONIC@359..362 "and"
      WHITESPACE@362..366 "    "
      NUMBER@366..385 "$0xfffffffffffffff0"
      COMMA@385..386 ","
      REGISTER@386..390 "%rsp"
    WHITESPACE@390..391 "\n"
"#
    );
}

#[test]
fn test_objdump_hex_first() {
    assert_listing!(
        r#"
target/debug/tests:	file format mach-o arm64


Disassembly of section .text:

10000102c: 9d 03 00 94 	bl	0x100001ea0 <__ZN4core3ptr6unique15Unique$LT$T$GT$6as_ptr17h08df014b8a084fd5E>"#,
        r#"ROOT@0..184
  WHITESPACE@0..1 "\n"
  METADATA@1..45 "target/debug/tests:\tf ..."
  WHITESPACE@45..48 "\n\n\n"
  METADATA@48..77 "Disassembly of sectio ..."
  WHITESPACE@77..79 "\n\n"
  INSTRUCTION@79..184
    METADATA@79..88 "10000102c"
    METADATA@88..89 ":"
    WHITESPACE@89..90 " "
    METADATA@90..102 "9d 03 00 94 "
    WHITESPACE@102..103 "\t"
    MNEMONIC@103..105 "bl"
    WHITESPACE@105..106 "\t"
    NUMBER@106..117 "0x100001ea0"
    WHITESPACE@117..118 " "
    METADATA@118..184
      BRACKETS@118..184
        L_ANGLE@118..119 "<"
        TOKEN@119..183 "__ZN4core3ptr6unique1 ..."
        R_ANGLE@183..184 ">"
"#
    );
}

#[test]
fn test_objdump_plus() {
    assert_listing!(
        r#"
a.out:     file format elf64-littleaarch64


Disassembly of section .text:

  210640:	b4000040 	cbz	x0, 210648 <call_weak_fn+0x10>"#,
        r#"ROOT@0..131
  WHITESPACE@0..1 "\n"
  METADATA@1..43 "a.out:     file forma ..."
  WHITESPACE@43..46 "\n\n\n"
  METADATA@46..75 "Disassembly of sectio ..."
  WHITESPACE@75..79 "\n\n  "
  INSTRUCTION@79..131
    METADATA@79..85 "210640"
    METADATA@85..86 ":"
    WHITESPACE@86..87 "\t"
    METADATA@87..96 "b4000040 "
    WHITESPACE@96..97 "\t"
    MNEMONIC@97..100 "cbz"
    WHITESPACE@100..101 "\t"
    REGISTER@101..103 "x0"
    COMMA@103..104 ","
    WHITESPACE@104..105 " "
    NUMBER@105..111 "210648"
    WHITESPACE@111..112 " "
    METADATA@112..131
      BRACKETS@112..131
        L_ANGLE@112..113 "<"
        TOKEN@113..125 "call_weak_fn"
        OPERATOR@125..126 "+"
        NUMBER@126..130 "0x10"
        R_ANGLE@130..131 ">"
"#
    );
}
