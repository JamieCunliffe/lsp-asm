.include "file_b.s"

label:
    stp a_register, x21, [sp, #imm_equ]!
    b fileb_label
    b .local_test
.local_test:
