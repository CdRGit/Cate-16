SDIR = src
ODIR = obj

OBJS = $(patsubst %.asm, %.o, $(wildcard $(SDIR)/*.asm))
OBJS := $(subst $(SDIR), $(ODIR), $(OBJS))

boot_rom: bin/rom
	cat $(OBJS) > $@

bin/rom:$(OBJS)
	ld65 -o bin/rom -C cate-16_layout.cfg $(OBJS)

obj/%.o: src/%.asm
	ca65 -o $@ $< --cpu 65816 --smart --feature string_escapes --feature underline_in_numbers