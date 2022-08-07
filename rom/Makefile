boot_rom: bin/rom
	cat bin/rom-* > $@

bin/rom: obj/*.o
	ld65 -o bin/rom -C cate-16_layout.cfg obj/*.o

obj/%.o: src/%.asm
	ca65 -o $@ $? --cpu 65816