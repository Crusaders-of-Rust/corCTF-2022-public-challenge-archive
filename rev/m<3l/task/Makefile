.PHONY := clean

'm<3l.exe': src.asm
	nasm -f bin src.asm -o $@
	chmod +x $@

clean:
	rm -f 'm<3l.exe'
