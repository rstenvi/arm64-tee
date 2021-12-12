#include "arm64.h"
OUTPUT_ARCH(aarch64)
ENTRY(_start)

SECTIONS {
	. = LOAD_ADDR;
	.text : {
		*(.init)
		*(.text*)
	}
	.data : ALIGN(4096) {
		*(.data*)
	}
	.bss : ALIGN(4096) {
		*(.bss*)
	}
	.rodata : ALIGN(4096) {
		*(.rodata*)
	}
}
