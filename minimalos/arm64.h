#ifndef __ARM64_H
#define __ARM64_H

//#define LOAD_ADDR 0x40080000
#define LOAD_ADDR 0x60000000
#define CPACR_EL1_FPEN (0b11 << 20)

#define SMCC_FAST  (1UL << 31)
#define SMCC_YIELD (0UL)
#define SMCC_64    (1UL << 30)
#define SMCC_32    (0UL)
#define SMCC_OEN_SHIFT (24)
#define SMCC_OEN_BITS  (6)
#define SMCC_OEN_MASK  ((1UL << SMCC_OEN_BITS) - 1)

#define TSP_ID    (0x32UL)
#define OPTEE_ID  (0x3eUL)
#define CUSTOM_ID (0x3dUL)

#define SMCC_OEN(id) ((id & SMCC_OEN_MASK) << SMCC_OEN_SHIFT)
#define SMCC_FNID(id) (id & 0xffffUL)


#define SMCC_FAST64(oen, id) (SMCC_FAST | SMCC_64 | SMCC_OEN(oen) | SMCC_FNID(id))
#define SMCC_FAST32(oen, id) (SMCC_FAST | SMCC_32 | SMCC_OEN(oen) | SMCC_FNID(id))
#define SMCC_YIELD64(oen, id) (SMCC_YIELD | SMCC_64 | SMCC_OEN(oen) | SMCC_FNID(id))
#define SMCC_YIELD32(oen, id) (SMCC_YIELD | SMCC_32 | SMCC_OEN(oen) | SMCC_FNID(id))

#endif
