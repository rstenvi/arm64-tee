ARCH := arm64
BUILD := debug
NAME := bl32
LIBOBJ := target/target/$(BUILD)/lib$(NAME).a

# We must provide an address of where we're loaded in memory. OS will work if
# it's incorrent, but we must then offset function pointers used, for instance
# when returning to EL0. If address is unknown, it should be set to 0.
LOAD_ENTRY_BL32 := 0xe100000


ifeq ($(ARCH),arm64)
	CROSS_COMPILE := aarch64-linux-gnu-
	TARGET := aarch64-linux-gnu-
endif

CC := $(CROSS_COMPILE)gcc
AS := $(CROSS_COMPILE)gcc
LD := $(CROSS_COMPILE)ld
OBJDUMP := $(CROSS_COMPILE)objdump
OBJCOPY := $(CROSS_COMPILE)objcopy

RUST_OPTS += --cfg gic="2"
#RUST_OPTS += --cfg spd="tsp"
RUST_OPTS += --cfg spd="optee"
RUST_OPTS += --cfg platform="qemu"
RUST_OPTS += -C soft-float
RUST_OPTS += -C panic=abort
RUST_OPTS += -C opt-level=z
RUST_OPTS += -C debug-assertions=off
RUST_OPTS += -C lto=no
#RUST_OPTS += -C codegen-units=1

ifeq ($(BUILD),debug)
	RUST_OPTS += -C debuginfo=2
endif

