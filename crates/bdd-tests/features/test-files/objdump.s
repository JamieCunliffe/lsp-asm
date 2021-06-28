
a.out:     file format elf64-x86-64


Disassembly of section .init:

0000000000401000 <_init>:
  401000:	f3 0f 1e fa          	endbr64 
  401004:	48 83 ec 08          	sub    $0x8,%rsp
  401008:	48 8b 05 e9 2f 00 00 	mov    0x2fe9(%rip),%rax        # 403ff8 <__gmon_start__>
  40100f:	48 85 c0             	test   %rax,%rax
  401012:	74 02                	je     401016 <_init+0x16>
  401014:	ff d0                	callq  *%rax
  401016:	48 83 c4 08          	add    $0x8,%rsp
  40101a:	c3                   	retq   

Disassembly of section .text:

0000000000401020 <_start>:
  401020:	f3 0f 1e fa          	endbr64 
  401024:	31 ed                	xor    %ebp,%ebp
  401026:	49 89 d1             	mov    %rdx,%r9
  401029:	5e                   	pop    %rsi
  40102a:	48 89 e2             	mov    %rsp,%rdx
  40102d:	48 83 e4 f0          	and    $0xfffffffffffffff0,%rsp
  401031:	50                   	push   %rax
  401032:	54                   	push   %rsp
  401033:	49 c7 c0 00 12 40 00 	mov    $0x401200,%r8
  40103a:	48 c7 c1 90 11 40 00 	mov    $0x401190,%rcx
  401041:	48 c7 c7 10 11 40 00 	mov    $0x401110,%rdi
  401048:	ff 15 a2 2f 00 00    	callq  *0x2fa2(%rip)        # 403ff0 <__libc_start_main@GLIBC_2.2.5>
  40104e:	f4                   	hlt    
  40104f:	90                   	nop

0000000000401050 <_dl_relocate_static_pie>:
  401050:	f3 0f 1e fa          	endbr64 
  401054:	c3                   	retq   
  401055:	66 2e 0f 1f 84 00 00 	nopw   %cs:0x0(%rax,%rax,1)
  40105c:	00 00 00 
  40105f:	90                   	nop

0000000000401060 <deregister_tm_clones>:
  401060:	b8 28 40 40 00       	mov    $0x404028,%eax
  401065:	48 3d 28 40 40 00    	cmp    $0x404028,%rax
  40106b:	74 13                	je     401080 <deregister_tm_clones+0x20>
  40106d:	b8 00 00 00 00       	mov    $0x0,%eax
  401072:	48 85 c0             	test   %rax,%rax
  401075:	74 09                	je     401080 <deregister_tm_clones+0x20>
  401077:	bf 28 40 40 00       	mov    $0x404028,%edi
  40107c:	ff e0                	jmpq   *%rax
  40107e:	66 90                	xchg   %ax,%ax
  401080:	c3                   	retq   
  401081:	66 66 2e 0f 1f 84 00 	data16 nopw %cs:0x0(%rax,%rax,1)
  401088:	00 00 00 00 
  40108c:	0f 1f 40 00          	nopl   0x0(%rax)

0000000000401090 <register_tm_clones>:
  401090:	be 28 40 40 00       	mov    $0x404028,%esi
  401095:	48 81 ee 28 40 40 00 	sub    $0x404028,%rsi
  40109c:	48 89 f0             	mov    %rsi,%rax
  40109f:	48 c1 ee 3f          	shr    $0x3f,%rsi
  4010a3:	48 c1 f8 03          	sar    $0x3,%rax
  4010a7:	48 01 c6             	add    %rax,%rsi
  4010aa:	48 d1 fe             	sar    %rsi
  4010ad:	74 11                	je     4010c0 <register_tm_clones+0x30>
  4010af:	b8 00 00 00 00       	mov    $0x0,%eax
  4010b4:	48 85 c0             	test   %rax,%rax
  4010b7:	74 07                	je     4010c0 <register_tm_clones+0x30>
  4010b9:	bf 28 40 40 00       	mov    $0x404028,%edi
  4010be:	ff e0                	jmpq   *%rax
  4010c0:	c3                   	retq   
  4010c1:	66 66 2e 0f 1f 84 00 	data16 nopw %cs:0x0(%rax,%rax,1)
  4010c8:	00 00 00 00 
  4010cc:	0f 1f 40 00          	nopl   0x0(%rax)

00000000004010d0 <__do_global_dtors_aux>:
  4010d0:	f3 0f 1e fa          	endbr64 
  4010d4:	80 3d 4d 2f 00 00 00 	cmpb   $0x0,0x2f4d(%rip)        # 404028 <__TMC_END__>
  4010db:	75 13                	jne    4010f0 <__do_global_dtors_aux+0x20>
  4010dd:	55                   	push   %rbp
  4010de:	48 89 e5             	mov    %rsp,%rbp
  4010e1:	e8 7a ff ff ff       	callq  401060 <deregister_tm_clones>
  4010e6:	c6 05 3b 2f 00 00 01 	movb   $0x1,0x2f3b(%rip)        # 404028 <__TMC_END__>
  4010ed:	5d                   	pop    %rbp
  4010ee:	c3                   	retq   
  4010ef:	90                   	nop
  4010f0:	c3                   	retq   
  4010f1:	66 66 2e 0f 1f 84 00 	data16 nopw %cs:0x0(%rax,%rax,1)
  4010f8:	00 00 00 00 
  4010fc:	0f 1f 40 00          	nopl   0x0(%rax)

0000000000401100 <frame_dummy>:
  401100:	f3 0f 1e fa          	endbr64 
  401104:	eb 8a                	jmp    401090 <register_tm_clones>
  401106:	66 2e 0f 1f 84 00 00 	nopw   %cs:0x0(%rax,%rax,1)
  40110d:	00 00 00 

0000000000401110 <main>:
  401110:	55                   	push   %rbp
  401111:	48 89 e5             	mov    %rsp,%rbp
  401114:	31 c0                	xor    %eax,%eax
  401116:	c7 45 fc 00 00 00 00 	movl   $0x0,-0x4(%rbp)
  40111d:	89 7d f8             	mov    %edi,-0x8(%rbp)
  401120:	48 89 75 f0          	mov    %rsi,-0x10(%rbp)
  401124:	5d                   	pop    %rbp
  401125:	c3                   	retq   
  401126:	66 2e 0f 1f 84 00 00 	nopw   %cs:0x0(%rax,%rax,1)
  40112d:	00 00 00 

0000000000401130 <sum>:
  401130:	55                   	push   %rbp
  401131:	48 89 e5             	mov    %rsp,%rbp
  401134:	48 89 7d f8          	mov    %rdi,-0x8(%rbp)
  401138:	48 89 75 f0          	mov    %rsi,-0x10(%rbp)
  40113c:	0f 57 c0             	xorps  %xmm0,%xmm0
  40113f:	f3 0f 11 45 ec       	movss  %xmm0,-0x14(%rbp)
  401144:	48 c7 45 e0 00 00 00 	movq   $0x0,-0x20(%rbp)
  40114b:	00 
  40114c:	48 8b 45 e0          	mov    -0x20(%rbp),%rax
  401150:	48 3b 45 f0          	cmp    -0x10(%rbp),%rax
  401154:	0f 8d 2a 00 00 00    	jge    401184 <sum+0x54>
  40115a:	48 8b 45 f8          	mov    -0x8(%rbp),%rax
  40115e:	48 8b 4d e0          	mov    -0x20(%rbp),%rcx
  401162:	f3 0f 10 04 88       	movss  (%rax,%rcx,4),%xmm0
  401167:	f3 0f 58 45 ec       	addss  -0x14(%rbp),%xmm0
  40116c:	f3 0f 11 45 ec       	movss  %xmm0,-0x14(%rbp)
  401171:	48 8b 45 e0          	mov    -0x20(%rbp),%rax
  401175:	48 05 01 00 00 00    	add    $0x1,%rax
  40117b:	48 89 45 e0          	mov    %rax,-0x20(%rbp)
  40117f:	e9 c8 ff ff ff       	jmpq   40114c <sum+0x1c>
  401184:	f3 0f 10 05 78 0e 00 	movss  0xe78(%rip),%xmm0        # 402004 <_IO_stdin_used+0x4>
  40118b:	00 
  40118c:	5d                   	pop    %rbp
  40118d:	c3                   	retq   
  40118e:	66 90                	xchg   %ax,%ax

0000000000401190 <__libc_csu_init>:
  401190:	f3 0f 1e fa          	endbr64 
  401194:	41 57                	push   %r15
  401196:	4c 8d 3d b3 2c 00 00 	lea    0x2cb3(%rip),%r15        # 403e50 <__frame_dummy_init_array_entry>
  40119d:	41 56                	push   %r14
  40119f:	49 89 d6             	mov    %rdx,%r14
  4011a2:	41 55                	push   %r13
  4011a4:	49 89 f5             	mov    %rsi,%r13
  4011a7:	41 54                	push   %r12
  4011a9:	41 89 fc             	mov    %edi,%r12d
  4011ac:	55                   	push   %rbp
  4011ad:	48 8d 2d a4 2c 00 00 	lea    0x2ca4(%rip),%rbp        # 403e58 <__do_global_dtors_aux_fini_array_entry>
  4011b4:	53                   	push   %rbx
  4011b5:	4c 29 fd             	sub    %r15,%rbp
  4011b8:	48 83 ec 08          	sub    $0x8,%rsp
  4011bc:	e8 3f fe ff ff       	callq  401000 <_init>
  4011c1:	48 c1 fd 03          	sar    $0x3,%rbp
  4011c5:	74 1f                	je     4011e6 <__libc_csu_init+0x56>
  4011c7:	31 db                	xor    %ebx,%ebx
  4011c9:	0f 1f 80 00 00 00 00 	nopl   0x0(%rax)
  4011d0:	4c 89 f2             	mov    %r14,%rdx
  4011d3:	4c 89 ee             	mov    %r13,%rsi
  4011d6:	44 89 e7             	mov    %r12d,%edi
  4011d9:	41 ff 14 df          	callq  *(%r15,%rbx,8)
  4011dd:	48 83 c3 01          	add    $0x1,%rbx
  4011e1:	48 39 dd             	cmp    %rbx,%rbp
  4011e4:	75 ea                	jne    4011d0 <__libc_csu_init+0x40>
  4011e6:	48 83 c4 08          	add    $0x8,%rsp
  4011ea:	5b                   	pop    %rbx
  4011eb:	5d                   	pop    %rbp
  4011ec:	41 5c                	pop    %r12
  4011ee:	41 5d                	pop    %r13
  4011f0:	41 5e                	pop    %r14
  4011f2:	41 5f                	pop    %r15
  4011f4:	c3                   	retq   
  4011f5:	66 66 2e 0f 1f 84 00 	data16 nopw %cs:0x0(%rax,%rax,1)
  4011fc:	00 00 00 00 

0000000000401200 <__libc_csu_fini>:
  401200:	f3 0f 1e fa          	endbr64 
  401204:	c3                   	retq   

Disassembly of section .fini:

0000000000401208 <_fini>:
  401208:	f3 0f 1e fa          	endbr64 
  40120c:	48 83 ec 08          	sub    $0x8,%rsp
  401210:	48 83 c4 08          	add    $0x8,%rsp
  401214:	c3                   	retq   
