include ../../config.mk

CFLAGS += -DSPD=OPTEE

all: bl32.o bl32.lds

bl32.lds: linker.lds.S
	-rm -f $@
	$(CC) -E -nostdinc -DEL=1 -DIMAGE_LOAD=$(LOAD_ENTRY_BL32) $< -o $@
	sed -i '/#.*/d' $@
	chmod 400 $@

bl32.o: arch.S
	$(AS) $(CFLAGS) -DIMAGE_LOAD=$(LOAD_ENTRY_BL32) -DEL=1 -c -o $@ $<


DEPS := $(OBJ:.o=.d)
-include $(DEPS)

clean:
	-rm -f $(OBJ) *.lds *.d *.o
