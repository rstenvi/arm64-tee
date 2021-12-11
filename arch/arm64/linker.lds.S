#include "aarch64.h"

OUTPUT_ARCH(aarch64)
ENTRY(_start)

SECTIONS {
	. = IMAGE_LOAD;
	IMAGE_START = .;
	.text : ALIGN(ARM64_PAGE_SIZE) {
		TEXT_START = .;
		*(.init)
		*(.text*)
		TEXT_STOP = .;
	}
	.rodata : ALIGN(ARM64_PAGE_SIZE) {
		RODATA_START = .;
		*(.rodata*)
		*(.eh_frame)
		RODATA_STOP = .;
	}
	.data : ALIGN(ARM64_PAGE_SIZE) {
		DATA_START = .;
		*(.data*)
	}
	.bss : ALIGN(ARM64_PAGE_SIZE) {
		*(.bss*)
		. = ALIGN(ARM64_PAGE_SIZE);
		KSTACK_INIT_LOW = .;
		. += ARM64_PAGE_SIZE;
		PGD_EL1 = .;
		. += ARM64_PAGE_SIZE;
		DATA_STOP = .;
	}
	IMAGE_END = .;
}

