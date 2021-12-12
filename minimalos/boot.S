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



	/* Enter endless loop */
1:
	b 1b
