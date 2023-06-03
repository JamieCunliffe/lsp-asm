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

#[test]
fn test_objdump_directive() {
    assert_listing!(
        r#"
a.out:     file format elf64-littleaarch64


Disassembly of section .text:

00000000002105e8 <_start>:
  210cac:	00000000 	.inst	0x00000000 ; undefined
"#,
        r#"ROOT@0..153
  WHITESPACE@0..1 "\n"
  METADATA@1..43 "a.out:     file forma ..."
  WHITESPACE@43..46 "\n\n\n"
  METADATA@46..75 "Disassembly of sectio ..."
  WHITESPACE@75..77 "\n\n"
  LABEL@77..153
    METADATA@77..93 "00000000002105e8"
    WHITESPACE@93..94 " "
    LABEL@94..103 "<_start>:"
    WHITESPACE@103..106 "\n  "
    DIRECTIVE@106..152
      METADATA@106..112 "210cac"
      METADATA@112..113 ":"
      WHITESPACE@113..114 "\t"
      METADATA@114..123 "00000000 "
      WHITESPACE@123..124 "\t"
      MNEMONIC@124..129 ".inst"
      WHITESPACE@129..130 "\t"
      NUMBER@130..140 "0x00000000"
      WHITESPACE@140..141 " "
      COMMENT@141..152 "; undefined"
    WHITESPACE@152..153 "\n"
"#
    );
}

#[test]
fn test_objdump_osx() {
    assert_listing!(
        r#"
a.out:	file format mach-o arm64

Disassembly of section __TEXT,__text:

0000000100003fa4 <_main>:
100003fa4: ff 43 00 d1 	sub	sp, sp, #16"#,
        r##"ROOT@0..138
  WHITESPACE@0..1 "\n"
  METADATA@1..32 "a.out:\tfile format ma ..."
  WHITESPACE@32..34 "\n\n"
  METADATA@34..71 "Disassembly of sectio ..."
  WHITESPACE@71..73 "\n\n"
  LABEL@73..138
    METADATA@73..89 "0000000100003fa4"
    WHITESPACE@89..90 " "
    LABEL@90..98 "<_main>:"
    WHITESPACE@98..99 "\n"
    INSTRUCTION@99..138
      METADATA@99..108 "100003fa4"
      METADATA@108..109 ":"
      WHITESPACE@109..110 " "
      METADATA@110..122 "ff 43 00 d1 "
      WHITESPACE@122..123 "\t"
      MNEMONIC@123..126 "sub"
      WHITESPACE@126..127 "\t"
      REGISTER@127..129 "sp"
      COMMA@129..130 ","
      WHITESPACE@130..131 " "
      REGISTER@131..133 "sp"
      COMMA@133..134 ","
      WHITESPACE@134..135 " "
      IMMEDIATE@135..136 "#"
      NUMBER@136..138 "16"
"##
    );
}

#[test]
fn test_llvm_objdump() {
    assert_listing!(
        r##"
target/release/test:	file format elf64-x86-64

Disassembly of section .init:

0000000000055000 <_init>:
   55000: f3 0f 1e fa                  	endbr64
"##,
        r##"ROOT@0..153
  WHITESPACE@0..1 "\n"
  METADATA@1..46 "target/release/test:\t ..."
  WHITESPACE@46..48 "\n\n"
  METADATA@48..77 "Disassembly of sectio ..."
  WHITESPACE@77..79 "\n\n"
  LABEL@79..153
    METADATA@79..95 "0000000000055000"
    WHITESPACE@95..96 " "
    LABEL@96..104 "<_init>:"
    WHITESPACE@104..108 "\n   "
    INSTRUCTION@108..152
      METADATA@108..113 "55000"
      METADATA@113..114 ":"
      WHITESPACE@114..115 " "
      METADATA@115..144 "f3 0f 1e fa           ..."
      WHITESPACE@144..145 "\t"
      MNEMONIC@145..152 "endbr64"
    WHITESPACE@152..153 "\n"
"##
    );
}

#[test]
fn test_objdump_no_insr() {
    assert_listing!(
        r##"

target/release/test:	file format mach-o arm64

Disassembly of section __TEXT,__text:

000000010002ad84 <__ZN4core3ptr100drop_in_place$LT$core..option..Option$LT$lsp_types..completion..CompletionClientCapabilities$GT$$GT$17h599609cf084734b7E>:
10002ad84:     	ldr	x8, [x0]
10002ad88:     	cmp	x8, #2
10002ad8c:     	b.ne	0x10002ad94 <__ZN4core3ptr100drop_in_place$LT$core..option..Option$LT$lsp_types..completion..CompletionClientCapabilities$GT$$GT$17h599609cf084734b7E+0x10>
10002ad90:     	ret"##,
        r##"ROOT@0..497
  WHITESPACE@0..2 "\n\n"
  METADATA@2..47 "target/release/test:\t ..."
  WHITESPACE@47..49 "\n\n"
  METADATA@49..86 "Disassembly of sectio ..."
  WHITESPACE@86..88 "\n\n"
  LABEL@88..497
    METADATA@88..104 "000000010002ad84"
    WHITESPACE@104..105 " "
    LABEL@105..244 "<__ZN4core3ptr100drop ..."
    WHITESPACE@244..245 "\n"
    INSTRUCTION@245..273
      METADATA@245..254 "10002ad84"
      METADATA@254..255 ":"
      WHITESPACE@255..261 "     \t"
      MNEMONIC@261..264 "ldr"
      WHITESPACE@264..265 "\t"
      REGISTER@265..267 "x8"
      COMMA@267..268 ","
      WHITESPACE@268..269 " "
      BRACKETS@269..273
        L_SQ@269..270 "["
        REGISTER@270..272 "x0"
        R_SQ@272..273 "]"
    WHITESPACE@273..274 "\n"
    INSTRUCTION@274..300
      METADATA@274..283 "10002ad88"
      METADATA@283..284 ":"
      WHITESPACE@284..290 "     \t"
      MNEMONIC@290..293 "cmp"
      WHITESPACE@293..294 "\t"
      REGISTER@294..296 "x8"
      COMMA@296..297 ","
      WHITESPACE@297..298 " "
      IMMEDIATE@298..299 "#"
      NUMBER@299..300 "2"
    WHITESPACE@300..301 "\n"
    INSTRUCTION@301..477
      METADATA@301..310 "10002ad8c"
      METADATA@310..311 ":"
      WHITESPACE@311..317 "     \t"
      MNEMONIC@317..321 "b.ne"
      WHITESPACE@321..322 "\t"
      NUMBER@322..333 "0x10002ad94"
      WHITESPACE@333..334 " "
      METADATA@334..477
        BRACKETS@334..477
          L_ANGLE@334..335 "<"
          TOKEN@335..471 "__ZN4core3ptr100drop_ ..."
          OPERATOR@471..472 "+"
          NUMBER@472..476 "0x10"
          R_ANGLE@476..477 ">"
    WHITESPACE@477..478 "\n"
    INSTRUCTION@478..497
      METADATA@478..487 "10002ad90"
      METADATA@487..488 ":"
      WHITESPACE@488..494 "     \t"
      MNEMONIC@494..497 "ret"
"##
    );
}

#[test]
fn objdump_no_leading_addr() {
    assert_listing!(
        r#"
lsp-asm:	file format mach-o 64-bit x86-64

Disassembly of section __TEXT,__text:

<__ZN146_$LT$alloc..boxed..Box$LT$dyn$u20$core..error..Error$u2b$core..marker..Sync$u2b$core..marker..Send$GT$$u20$as$u20$core..convert..From$LT$E$GT$$GT$4from17h2f705b76ff6935a5E>:
 55                                    	pushq	%rbp
 48 89 e5                              	movq	%rsp, %rbp
"#,
        r#"ROOT@0..372
  WHITESPACE@0..1 "\n"
  METADATA@1..42 "lsp-asm:\tfile format  ..."
  WHITESPACE@42..44 "\n\n"
  METADATA@44..81 "Disassembly of sectio ..."
  WHITESPACE@81..83 "\n\n"
  LABEL@83..372
    LABEL@83..264 "<__ZN146_$LT$alloc..b ..."
    WHITESPACE@264..266 "\n "
    INSTRUCTION@266..315
      METADATA@266..304 "55                    ..."
      WHITESPACE@304..305 "\t"
      MNEMONIC@305..310 "pushq"
      WHITESPACE@310..311 "\t"
      REGISTER@311..315 "%rbp"
    WHITESPACE@315..317 "\n "
    INSTRUCTION@317..371
      METADATA@317..355 "48 89 e5              ..."
      WHITESPACE@355..356 "\t"
      MNEMONIC@356..360 "movq"
      WHITESPACE@360..361 "\t"
      REGISTER@361..365 "%rsp"
      COMMA@365..366 ","
      WHITESPACE@366..367 " "
      REGISTER@367..371 "%rbp"
    WHITESPACE@371..372 "\n"
"#
    );
}
