#ifndef __AARCH64_H
#define __AARCH64_H

#define TSP_ENTRY_DONE         0xf2000000
#define TSP_ON_DONE            0xf2000001
#define TSP_OFF_DONE           0xf2000002
#define TSP_SUSPEND_DONE       0xf2000003
#define TSP_RESUME_DONE        0xf2000004
#define TSP_PREEMPTED          0xf2000005
#define TSP_ABORT_DONE         0xf2000007
#define TSP_SYSTEM_OFF_DONE    0xf2000008
#define TSP_SYSTEM_RESET_DONE  0xf2000009


#define SMCC_32    (0)
#define SMCC_FAST  (1)
#define OPTEE_ID(fnid)  (SMCC_FAST << 31 | SMCC_32 << 30 | 62 << 24 | fnid)

#define OPTEE_ENTRY_DONE OPTEE_ID(0)

//#ifndef MAX_CPUS
//# define MAX_CPUS 2
//#endif

//#define IMAGE_LOAD 0x42000


#define ARM64_PAGE_SIZE (4096)
#define PAGE_SIZE (ARM64_PAGE_SIZE)

//#define STACK_SIZE_ALLOCATED (PAGE_SIZE * MAX_CPUS)

#define EXC_EXC_SP_OFFSET 16
#define AARCH64_EXC_SYNC_SP0      0x1
#define AARCH64_EXC_IRQ_SP0       0x2
#define AARCH64_EXC_FIQ_SP0       0x3
#define AARCH64_EXC_SERR_SP0      0x4

#define AARCH64_EXC_SYNC_SPX      0x11
#define AARCH64_EXC_IRQ_SPX       0x12
#define AARCH64_EXC_FIQ_SPX       0x13
#define AARCH64_EXC_SERR_SPX      0x14

#define AARCH64_EXC_SYNC_AARCH64  0x21
#define AARCH64_EXC_IRQ_AARCH64   0x22
#define AARCH64_EXC_FIQ_AARCH64   0x23
#define AARCH64_EXC_SERR_AARCH64  0x24

#define AARCH64_EXC_SYNC_AARCH32  0x31
#define AARCH64_EXC_IRQ_AARCH32   0x32
#define AARCH64_EXC_FIQ_AARCH32   0x33
#define AARCH64_EXC_SERR_AARCH32  0x34

#define DAIF_FIQ_BIT		(1 << 0)
#define DAIF_IRQ_BIT		(1 << 1)
#define DAIF_ABT_BIT		(1 << 2)


#if EL == 1
# define VBAR_ELx  vbar_el1
# define SCTLR_ELx sctlr_el1
# define MPIDR_ELx mpidr_el1
# define SPSR_ELx  spsr_el1
# define ELR_ELx   elr_el1
# define SCR_ELx   scr_el1
# define SP_LOWER  sp_el0
# define ESR_ELx   esr_el1
#elif EL == 3
# define VBAR_ELx  vbar_el3
# define SCTLR_ELx sctlr_el3
# define MPIDR_ELx mpidr_el1
# define SPSR_ELx  spsr_el3
# define ELR_ELx   elr_el3
# define SCR_ELx   scr_el3
# define SP_LOWER  sp_el1
# define ESR_ELx   esr_el3
#else
# error "Invalid EL
#endif

#define TSP   (1)
#define OPTEE (2)


#define OFFSET_SPSR (8)
#define OFFSET_ELR  (0)
#define OFFSET_SCR  (16)
#define OFFSET_REGS (32)


#endif
