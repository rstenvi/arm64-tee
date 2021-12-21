#include "arm64.h"

.section .init
_header:
	add x13, x18, #0x16
	b _start
	.quad 0x0
	.quad 0x0

	/* LE | 4KB pages | 2MB aligned */
	.quad 0b1010

	.quad 0x0
	.quad 0x0
	.quad 0x0
	.ascii "ARM\x64"
	.long 0

.section .text
_start:
	/* Store load address */
	adr x11, .

	/* Store address to FDT */
	mov x10, x0

	mov x0, #CPACR_EL1_FPEN
	msr cpacr_el1, x0

	# mov x0,#((1UL<<31)|(0UL)|(((0x3dUL)&((1UL<<(6))-1))<<(24))|(0&0xffffUL))
	mov x0, #(1 << 31)
	mov x11, 0x3d
	lsl x11, x11, #24
	add x0, x0, x11
	smc #0x0


	/* Enter endless loop */
1:
	b 1b
