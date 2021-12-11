include config.mk

COBJ = clib.o

all: $(NAME).bin

$(NAME).elf: $(LIBOBJ) deps $(COBJ)
	$(LD) -Tarch/$(ARCH)/$(NAME).lds -Map=$(NAME).map arch/$(ARCH)/$(NAME).o $(COBJ) $(OBJ) $< -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@

deps:
	make -C arch/$(ARCH)

$(LIBOBJ): PHONY Makefile arch/$(ARCH)/target.json
	RUSTFLAGS='--cfg arch__$(ARCH) $(RUST_OPTS)' cargo build -Z build-std=core,alloc --target=arch/$(ARCH)/target.json

clean:
	make -C arch/$(ARCH) clean
	-rm -f *.bin *.elf *.o $(NAME).map

fclean: clean
	-rm -rf target/

.PHONY: deps PHONY
