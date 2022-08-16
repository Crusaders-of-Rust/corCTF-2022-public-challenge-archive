  BITS 32

                org     0x100000                      ; will jump 0x47

  ehdr:                                                 ; Elf32_Ehdr
                db      0x7F, "ELF", 1, 1, 1, 0         ;   e_ident
        times 8 db      0
                dw      2                               ;   e_type
                dw      3                               ;   e_machine
                dd      1                               ;   e_version
                dd      _start                          ;   e_entry
                dd      phdr - $$                       ;   e_phoff
                dd      0                               ;   e_shoff
                dd      7                               ;   e_flags
                dw      ehdrsize                        ;   e_ehsize
                dw      phdrsize                        ;   e_phentsize
                dw      1                               ;   e_phnum
                dw      0                               ;   e_shentsize
                dw      0                               ;   e_shnum
                dw      0                               ;   e_shstrndx
  ehdrsize      equ     $ - ehdr

  phdr:                                                 ; Elf32_Phdr
                dd      1                               ;   p_type
                dd      0                               ;   p_offset
                dd      $$                              ;   p_vaddr
                dd      $$                              ;   p_paddr
                ; dd      filesize                        ;   p_filesz
                ; dd      filesize                        ;   p_memsz
                ; dd      0xcc000000                        ;   p_filesz
                ; dd      0xcc000000                        ;   p_memsz

                db 0x00
                db 0x00
                db 0x00
                jmp short $ + (label - $)
                db 0x00
                db 0x00
                db 0xeb

                ; dd 0x0064EB00
                ; dd 0x0064EB00
                dd      7                               ;   p_flags
                dd      0x1000                          ;   p_align

  phdrsize      equ     $ - phdr
 _start:        ; Entrypoint for ELF
                mov al, 3
                xor ebx, ebx      ; stdin
                mov ecx, buf      ; buffer
                mov dl,  255      ; len
                int 0x80          ; read(0, buf, 255)
 
 xor esi, esi
 elfloop:
 cmp esi, rotlen
 je elfcorrect
 mov al, byte [buf+esi]
 rol al, 13
 mov bl, byte [rotbuf+esi]
 cmp al, bl
 jne endelfloop
 inc si
 jmp elfloop
 
 elfcorrect:
 mov word [correct], 1
 endelfloop:
 mov ebx, 1
 mov eax, 4
 cmp word [correct], 1
 je elfwin
 mov ecx, elflosebuf
 mov edx, elfloselen
 int 0x80
jmp exit

 elfwin:
 mov ecx, elfwinbuf
 mov edx, elflosebuf - elfwinbuf
 int 0x80
exit:
mov eax, 1
mov ebx, 0
int 0x80

  filesize      equ     $ - $$


bits 16
label:          ; Entrypoint for DOS, add 0x100 to all addresses
mov ah, 0ah
lea dx, [buf+0x100]
mov byte [buf+0x100], 255
mov byte [buf+0x100+1], 255
int 21h         ; read up to 255 chars

xor si, si

dosloop:
cmp si, xorlen  ; check if end of buffer
je doscorrect
mov al, byte [buf+0x100+2+si]
xor al, 13
mov bl, byte [xorbuf+0x100+si]
cmp al, bl      ; if input is wrong, jump out 
jne enddosloop
inc si
jmp dosloop
doscorrect:
mov word [correct+0x100], 1
enddosloop:
xor bx, bx
mov ah, 9
cmp word [correct+0x100], 1
je doswin
doslose:
lea dx, [doslosebuf+0x100]
int 21h
mov ah, 4ch
int 21h

doswin:
lea dx, [doswinbuf+0x100]
int 21h
mov ah, 4ch
int 21h


buf: db 255
times 254 db 0
rotbuf: db 0x6c,0xed,0x4e,0x6c,0x8e,0xcc,0x6f,0x66,0xad,0x4c,0x4e,0x86,0x6c,0x66,0x85,0x66,0x0f,0x8e ; rol("corctf{3mbr4c3,3xt", 13)
rotlen equ $ - rotbuf
xorbuf: db 0x3e,0x63,0x69,0x21,0x3e,0x55,0x79,0x3c,0x63,0x6a,0x78,0x3c,0x38,0x65,0x2c,0x2c,0x3c,0x70  ; xor("3nd,3Xt1ngu15h!!1}", 13)
xorlen equ $ - xorbuf
correct: dw 0

doswinbuf: db 'Well done! Sadly, Linus Torvalds has embraced, extended and extinguished the other half of the flag :(', 0xa, 0xd, '$'
doslosebuf: db 'Incorrect :(', 0xa, 0xd, '$'
elfwinbuf: db 'Well done! Sadly, Microsoft has embraced, extended and extinguished the other half of the flag :(', 0xa
elflosebuf: db 'Incorrect :(', 0xa
elfloselen equ $ - elflosebuf
