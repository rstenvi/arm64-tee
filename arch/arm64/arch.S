#include "aarch64.h"

/* First code executed */
.global _start

/* Beginning of vectortable placed in VBAR_ELx */
.global vectortable

/* Shared exception handle after code in vectortable */
.global exception_handler

/* Return back to caller, either eret if we're in EL3 or smc if we're in EL1 */
.global exception_return

.extern rustmain



/*
* When we're called from TF-A, arguments are passed in x0-x7
* This macro can be used to store all arguments in a pointer in chosen register
*/
.macro PUSH_SMC_ARGS toreg
	stp x6, x7, [sp, #-16]!
	stp x4, x5, [sp, #-16]!
	stp x2, x3, [sp, #-16]!
	stp x0, x1, [sp, #-16]!

	mov \toreg, sp
.endm

/*
* When returning back to TF-A using smc we can pass arguments in x0-x7
* We can't return arguments like this using C or Rust, so can return a
* pointer in a register and load them using the macro below.
*/
.macro POP_SMC_ARGS
	ldp x0, x1, [sp], #16
	ldp x2, x3, [sp], #16
	ldp x4, x5, [sp], #16
	ldp x6, x7, [sp], #16
.endm

/*
* This code must be the first code in the file, so use .init as the section
* here. Linker-file will then ensure it's placed first in the .text section.
*/
.section .init
_start:

/* This could be used to load multiple OPTEE images */
/*
.word 0x4f505445
.byte 2
.byte 1
.short 0
.word 1

Addr high and low set to -1 allows firmware to choose address
.word 0xffffffff
.word 0xffffffff
.word 0
.word IMAGE_END - IMAGE_START
*/

	/* Save current PC so that we know were we've been loaded */
	adr x20, .

	/* Stave FDT in a register we're less likely to clobber */
	mov x19, x2

	/* Set up vector table */
	adr x1, vectortable
	msr VBAR_ELx, x1
	isb

	/*
	* We may be placed in S-EL2, currently not used, to just drop down to S-EL1
	*/
	bl drop_from_el2


	/*
	* Enable
	* - instruction cache
	* - stack pointer and data access alignement
	* Disable
	* - Speculative loads
	*/
	mrs x0, SCTLR_ELx
	mov x1, #((1 << 12) | (1 << 1) | (1 << 3))
	orr x0, x0, x1
	bic x0, x0, #(1 << 31)
	msr SCTLR_ELx, x0
	isb


	/* Clear mask for SError */
	msr daifclr, #(1 << 2)

	/*
	* Sanity check that we've been loaded on the address specified in linker
	* script.
	* TODO: Can support loading at different address, but need to adjust all
	* linker values.
	*/
	mov x1, #IMAGE_LOAD
	sub x2, x20, x1
	adr x3, loadoffset
	str x2, [x3]

/*	cmp x1, x20
	bne 2f
*/
	/* Set up a temporary stack from .bss segment */
	bl get_stack_area
	mov sp, x0

	/*
	* Prepare call to rustmain
	* - Set FDT as first argument
	* - Set load address and last image address as second and third argument
	*/
	mov x0, x19
	mov x1, x20
	adr x2, IMAGE_END
	bl rustmain

	/*
	* If non-zero is returned, we should halt execution.
	* Higher level code should report on what the error was
	*/
	cbnz x0, 2f

	/* Otherwise we should notify S-EL3 about where to find us again */

.global return_el3
return_el3:
#if SPD == TSP || SPD == OPTEE
	/*
	* Notify S-EL3 where they should jump on various
	* commands
	*/
	adr x1, eret_jmp_table
#if SPD == TSP
	mov x0, #TSP_ENTRY_DONE
#elif SPD == OPTEE
	mov x0, #(OPTEE_ENTRY_DONE)
#endif
	smc #0x0
#else
#error "No secure payload dispatcher was chosen"
#endif

/* Endless loop we end up in if error happens */
2:
	b 2b

/*
* Tables are aalmost equal for TSP and OPTEE
* OPTEE
struct optee_vectors {
    optee_vector_isn_t yield_smc_entry;
    optee_vector_isn_t fast_smc_entry;
    optee_vector_isn_t cpu_on_entry;
    optee_vector_isn_t cpu_off_entry;
    optee_vector_isn_t cpu_resume_entry;
    optee_vector_isn_t cpu_suspend_entry;
    optee_vector_isn_t fiq_entry;
    optee_vector_isn_t system_off_entry;
    optee_vector_isn_t system_reset_entry;
}
* TSP:
tsp_vectors {
    tsp_vector_isn_t yield_smc_entry;
    tsp_vector_isn_t fast_smc_entry;
    tsp_vector_isn_t cpu_on_entry;
    tsp_vector_isn_t cpu_off_entry;
    tsp_vector_isn_t cpu_resume_entry;
    tsp_vector_isn_t cpu_suspend_entry;
    tsp_vector_isn_t sel1_intr_entry;
    tsp_vector_isn_t system_off_entry;
    tsp_vector_isn_t system_reset_entry;
    tsp_vector_isn_t abort_yield_smc_entry;
}
*/
#if SPD == TSP || SPD == OPTEE
eret_jmp_table:
	b _smc_yield
	b _smc_fast
	b _smc_cpu_on
	b _smc_cpu_off
	b _smc_cpu_resume
	b _smc_cpu_suspend
#if SPD == TSP
	b _smc_sel1_interrupt
#elif SPD == OPTEE
	b _smc_fiq_entry
#endif
	b _smc_system_off
	b _smc_system_reset
#if SPD == TSP
	b _smc_abort_yield
#endif
#endif  /* SPD == TSP || SPD == OPTEE */


.macro ADJUST R1, R2, NAME
adr \R1, \NAME
add \R1, \R1, \R2
.endm

.section .text

.global arch_load_offset
arch_load_offset:
	adr x0, loadoffset
	ldr x0, [x0]
	ret

.global image_fill_map
image_fill_map:
	mov x2, xzr
	adr x1, TEXT_START
	str x1, [x0], #8
	adr x1, TEXT_STOP
	str x1, [x0], #8

	adr x1, RODATA_START
	str x1, [x0], #8
	adr x1, RODATA_STOP
	str x1, [x0], #8

	adr x1, DATA_START
	str x1, [x0], #8
	adr x1, DATA_STOP
	str x1, [x0], #8

	ret

/*
* exception_return expects the state stored on the stack, so this function
* copies the state from x0 onto the stack.
*/
.global drop_el0
drop_el0:
	/* Number of registers */
	mov x1, #36

	/* Go to end of struct */
	add x0, x0, #(8 * 36)
1:
	/* Start storing data and subtracting from stack */
	ldr x2, [x0, #-8]!
	str x2, [sp, #-8]!
	subs x1, x1, #1

	/* Branches when zero flag is set */
	b.ne 1b

	b exception_return
	ret

.global smcret
smcret:
	smc #0x0
	ret

.global hvc
hvc:
	hvc #0x0
	ret

.global arch_svc
arch_svc:
	/* Use x8 to reserve space for arguments in the future */
	mov x8, x3
	svc #0x0
	ret

halt:
	b halt

/*
* x0 = Top of new stack
*/
.global switch_stack
switch_stack:
	/* Store bottom of stack */
	sub x12, x0, #4096

	/* Current SP */
	mov x1, sp

	/* Get bottom of stack */
	and x0, x1, #~(4095)


	mov x2, xzr
	
	/* Pointer to new stack region */
	/*mov x3, x12 */
1:
	ldr x4, [x0], #8
	str x4, [x12], #8
	add x2, x2, #8
	cmp x2, #4096
	b.lt 1b


	/* Must place it at correct offset in stack */

	/* x0 is top of old stack and x12 is top of new stack */
	mov x1, sp
	sub x1, x0, x1
	sub x0, x12, x1
	mov sp, x0

	/* Offset on new stack */
	/* Save current offset in SP */
	/*
	sub x11, x1, x0
	add x0, x3, x11
	mov sp, x0
	*/
	ret

.global get_pgd_el1
get_pgd_el1:
	adr x0, PGD_EL1
	ret

get_temp_stack:
	adr x0, KSTACK_INIT_LOW
	add x0, x0, #PAGE_SIZE
	ret

get_stack_area:
	/* Get cpu ID */
	mrs x1, MPIDR_ELx
	and x1, x1, #0xff

	/* Multiply cpu id with page size to find appropriate section */
	mov x2, PAGE_SIZE
	mul x2, x2, x1

	/* Add offset to start of stack region */
	adr x0, KSTACK_INIT_LOW
	add x0, x0, x2

	/* Stack should be end of page */
	add x0, x0, PAGE_SIZE
	ret

drop_from_el2:
	mrs x0, CurrentEL
	lsr x0, x0, #2

	cmp x0, #1
	b.le 1f

	/*
	* TODO:
	* If we need access to EL2 registers, we can expose it here
	*/


	/* Clear M-bits and set EL1h */

	isb
	mrs x1, spsr_el2
	and x1, x1, #~(0b1111)
	orr x1, x1, #(1 << 2)	/* EL1 */
	orr x1, x1, #1			/* h */
	msr elr_el2, lr
	isb

	/* x0 contains previous EL */
	eret

1:
	/* x0 contains previous EL */
	ret

#if SPD == TSP || SPD == OPTEE
_smc_yield:
	msr daifclr, #DAIF_FIQ_BIT | DAIF_IRQ_BIT
	PUSH_SMC_ARGS x0
	bl smc_handler
	msr daifset, #DAIF_FIQ_BIT | DAIF_IRQ_BIT
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b

_smc_fast:
	PUSH_SMC_ARGS x0
	bl smc_handler
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b


_smc_cpu_off:
	PUSH_SMC_ARGS x0
	bl smc_cpu_off
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b

_smc_cpu_resume:
	PUSH_SMC_ARGS x0
	bl smc_cpu_resume
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b

_smc_cpu_suspend:
	PUSH_SMC_ARGS x0
	bl smc_cpu_suspend
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b


_smc_system_off:
	PUSH_SMC_ARGS x0
	bl smc_system_off
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b

_smc_system_reset:
	PUSH_SMC_ARGS x0
	bl smc_system_reset
	POP_SMC_ARGS
	smc #0x0

1:
	b 1b

/* Not supported */
_smc_abort_yield:
	b panic


#if SPD == OPTEE
_smc_fiq_entry:
	PUSH_SMC_ARGS x0
	bl smc_fiq_entry
	POP_SMC_ARGS
	smc #0x0
#endif

/*
* This is called to execute an S-EL1 interrupt while code was executing in normal
* world. x0 contains the magic number which indicate the type of interrupt.
*
*/
_smc_sel1_interrupt:
	b panic

_smc_cpu_on:

	adr x9, vectortable
	msr vbar_el1, x9
	isb

	msr daifclr, #(1 << 2)


	mrs x9, sctlr_el1
	mov x8, #((1 << 12) | (1 << 1) | (1 << 3))
	orr x9, x9, x8
	bic x9, x9, #(1 << 31)
	msr sctlr_el1, x9
	isb


	/* Set up a temporary stack to create a new stack */
	bl get_temp_stack
	mov sp, x0

	/* Should invalidate cache region */

	/* Get PGD in x0 and init virtual memory on CPU */
	bl get_pgd_el1
	bl mmu_init_cpu

	/* bl get_stack_area */
	bl get_new_stack
	mov sp, x0

	PUSH_SMC_ARGS x0

	bl smc_cpu_on

	POP_SMC_ARGS
	smc #0x0

1:
	b 1b
#endif  /* SPD == TSP || SPD == OPTEE */

exception_return:
	/* Skip type and esr */
	add sp, sp, #16

	/* saved_sp and elr */
	ldp x21, x22, [sp], #16
	msr sp_el0, x21
	msr elr_el1, x22

	/* spsr and x0 */
	ldp x21, x0, [sp], #16
	msr spsr_el1, x21

	/* Load all remaining GP regs */
	ldp x1, x2, [sp], #16
	ldp x3, x4, [sp], #16
	ldp x5, x6, [sp], #16
	ldp x7, x8, [sp], #16
	ldp x9, x10, [sp], #16
	ldp x11, x12, [sp], #16
	ldp x13, x14, [sp], #16
	ldp x15, x16, [sp], #16
	ldp x17, x18, [sp], #16
	ldp x19, x20, [sp], #16
	ldp x21, x22, [sp], #16
	ldp x23, x24, [sp], #16
	ldp x25, x26, [sp], #16
	ldp x27, x28, [sp], #16
	ldp x29, x30, [sp], #16

	/* We've changed lr, so we need to synchronize instruction buffer */
	isb
	eret


.macro vector_entry exceptiontype
	msr daifclr, #DAIF_ABT_BIT
	stp	x29, x30, [sp, #-16]!

	stp	x27, x28, [sp, #-16]!
	stp	x25, x26, [sp, #-16]!
	stp	x23, x24, [sp, #-16]!
	stp	x21, x22, [sp, #-16]!
	stp	x19, x20, [sp, #-16]!
	
	stp	x17, x18, [sp, #-16]!
	stp	x15, x16, [sp, #-16]!
	stp	x13, x14, [sp, #-16]!
	stp	x11, x12, [sp, #-16]!
	stp	x9, x10, [sp, #-16]!
	stp	x7, x8, [sp, #-16]!
	stp	x5, x6, [sp, #-16]!
	stp	x3, x4, [sp, #-16]!
	stp	x1, x2, [sp, #-16]!
	
	/* Get spsr value */
	mrs	x21, SPSR_ELx
	stp	x21, x0, [sp, #-16]!

	
	mrs	x21, ELR_ELx
	stp	xzr, x21, [sp, #-16]!

	/* Store exception type and esr */
	mov	x21, #(\exceptiontype)
	mrs	x22, ESR_ELx
	stp	x21, x22, [sp, #-16]!


	/* Store SP at appropriate offset */
	mrs	x21, SP_LOWER
	str	x21, [sp, EXC_EXC_SP_OFFSET]
	

	mov x0, sp
	/* Temporary */
	bl handle_exception

	b exception_return
.endm

.macro ALIGNED_BRANCH bxvalue
	.align 7
	b \bxvalue
.endm

.macro ALIGNED_ENTRY bxvalue num
.align 7
\bxvalue:
	vector_entry \num
.endm

.align 11
vectortable:
// Generate all the code
ALIGNED_ENTRY _curr_el_sp0_sync, AARCH64_EXC_SYNC_SP0
ALIGNED_ENTRY _curr_el_sp0_irq, AARCH64_EXC_IRQ_SP0
ALIGNED_ENTRY _curr_el_sp0_fiq, AARCH64_EXC_FIQ_SP0
ALIGNED_ENTRY _curr_el_sp0_serror, AARCH64_EXC_SERR_SP0
ALIGNED_ENTRY _curr_el_spx_sync, AARCH64_EXC_SYNC_SPX
ALIGNED_ENTRY _curr_el_spx_irq, AARCH64_EXC_IRQ_SPX
ALIGNED_ENTRY _curr_el_spx_fiq, AARCH64_EXC_FIQ_SPX
ALIGNED_ENTRY _curr_el_spx_serror, AARCH64_EXC_SERR_SPX
ALIGNED_ENTRY _lower_el_aarch64_sync, AARCH64_EXC_SYNC_AARCH64
ALIGNED_ENTRY _lower_el_aarch64_irq, AARCH64_EXC_IRQ_AARCH64
ALIGNED_ENTRY _lower_el_aarch64_fiq, AARCH64_EXC_FIQ_AARCH64
ALIGNED_ENTRY _lower_el_aarch64_serror, AARCH64_EXC_SERR_AARCH64
ALIGNED_ENTRY _lower_el_aarch32_sync, AARCH64_EXC_SYNC_AARCH32
ALIGNED_ENTRY _lower_el_aarch32_irq, AARCH64_EXC_IRQ_AARCH32
ALIGNED_ENTRY _lower_el_aarch32_fiq, AARCH64_EXC_FIQ_AARCH32
ALIGNED_ENTRY _lower_el_aarch32_serror, AARCH64_EXC_SERR_AARCH32

// Ensure we reserve enough bytes for last entry
.align 7


.section .data
loadoffset:
	.quad 0

