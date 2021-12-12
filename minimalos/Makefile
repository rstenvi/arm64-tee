# Should use dedicated cross compiler, but should be fine for this
CROSS ?= aarch64-linux-gnu-

CFLAGS += -nostdinc
CFLAGS += -MMD
LDFLAGS += -static

CC := $(CROSS)gcc
LD := $(CROSS)ld
AS := $(CROSS)gcc
OBJCOPY := $(CROSS)objcopy


OBJ += boot.o

all: Image

Image: minimalos.elf
	$(OBJCOPY) -O binary minimalos.elf Image	

minimalos.elf: $(OBJ) linker.lds
	$(LD) -T linker.lds -Map=System.map $(OBJ) $(LDFLAGS) -o $@

linker.lds: linker.lds.S
	-rm -f $@
	$(CC) -E -nostdinc $< -o $@
	awk '!/#.+/' $@ > $@.tmp
	mv $@.tmp $@
	rm -f $@.tmp
	chmod 400 $@

run: Image
	qemu-system-aarch64 -machine virt -cpu max -nographic -smp 1 -m 128M -kernel Image

clean:
	-rm -f linker.lds *.o Image minimalos.elf System.map
