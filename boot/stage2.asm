[org 0x7E00]
[bits 16]

stage2_entry:
    mov [boot_drive_s2], dl
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x9000
    mov si, msg_stage2_start
    call print_string_rm
    call enable_a20
    mov si, msg_a20_ok
    call print_string_rm
    mov si, msg_kernel_load
    call print_string_rm
    mov ax, 0x1000
    mov es, ax
    xor bx, bx
    mov byte [boot_drive_s2], dl
    mov cl, 3
    mov dh, 1
    mov ch, 0
    mov bp, 300
.load_kernel_loop:
    push cx
    mov ah, 0x02
    mov al, 1
    mov dl, [boot_drive_s2]
    int 0x13
    jc kernel_error
    pop cx
    add bx, 512
    cmp bx, 0
    jne .next_sector
    mov ax, es
    add ax, 0x1000
    mov es, ax
    xor bx, bx
.next_sector:
    inc cl
    cmp cl, 19
    jne .no_track_change
    mov cl, 1
    inc dh
    cmp dh, 2
    jne .no_head_change
    mov dh, 0
    inc ch
.no_head_change:
.no_track_change:
    dec bp
    jnz .load_kernel_loop
    mov si, msg_kernel_ok
    call print_string_rm
    xor ax, ax
    mov es, ax
    mov si, msg_gdt_load
    call print_string_rm
    lgdt [gdt_descriptor]
    mov ax, 0x4F00
    mov di, vbe_info_block
    int 0x10
    cmp ax, 0x004F
    jne vesa_error
    mov ax, word [vbe_info_block + 14] 
    mov fs, word [vbe_info_block + 16] 
    mov si, ax
    mov si, ax

vesa_find_mode:
    mov cx, fs:[si]
    cmp cx, 0xFFFF
    je vesa_error 
    mov ax, 0x4F01
    mov di, mode_info_block
    int 0x10
    cmp ax, 0x004F
    jne vesa_next_mode
    mov ax, word [mode_info_block]
    and ax, 0x80 
    jz vesa_next_mode
    mov ax, word [mode_info_block + 18] 
    cmp ax, 1920
    jne vesa_next_mode
    mov ax, word [mode_info_block + 20] 
    cmp ax, 1080
    jne vesa_next_mode
    mov al, byte [mode_info_block + 25] 
    cmp al, 32
    jne vesa_next_mode
    mov bx, cx
    or bx, 0x4000 
    mov ax, 0x4F02
    int 0x10
    cmp ax, 0x004F
    jne vesa_error
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov eax, dword [mode_info_block + 40] 
    mov dword [0x0800], eax
    mov dword [0x0804], 0
    mov ax, word [mode_info_block + 18] 
    mov word [0x0808], ax
    mov ax, word [mode_info_block + 20] 
    mov word [0x080A], ax
    mov ax, word [mode_info_block + 16] 
    mov word [0x080C], ax
    xor ax, ax
    mov al, byte [mode_info_block + 25] 
    mov word [0x080E], ax
    jmp vesa_success_init

vesa_next_mode:
    add si, 2
    jmp vesa_find_mode

vesa_error:
    mov si, msg_vesa_err
    call print_string_rm
    cli
    hlt

vesa_success_init:
    cli
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp CODE_SEG:init_pm

enable_a20:
    in al, 0x92
    test al, 2
    jnz .done
    or al, 2
    and al, 0xFE
    out 0x92, al
.done:
    ret

print_string_rm:
    lodsb
    or al, al
    jz .ret
    mov ah, 0x0E
    int 0x10
    jmp print_string_rm
.ret:
    ret
msg_stage2_start db "Stage 2 Loaded.", 13, 10, 0
msg_a20_ok       db "A20 loaded, ready to fire.", 13, 10, 0
msg_gdt_load     db "Loading GDT...", 13, 10, 0
msg_kernel_load  db "Praying that the kernel loads:", 13, 10, 0
msg_kernel_err   db "Kernel Load Error!", 13, 10, 0

kernel_error:
    mov si, msg_kernel_err
    call print_string_rm
    cli
    hlt
boot_drive_s2 db 0
align 8

gdt_start:
    dq 0x0000000000000000

gdt_code:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 11001111b
    db 0x00

gdt_data:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00

gdt_code64:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 10101111b
    db 0x00

gdt_data64:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 10101111b
    db 0x00

gdt_user_data64: 
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11110010b 
    db 10101111b
    db 0x00

gdt_user_code64: 
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11111010b 
    db 10101111b
    db 0x00

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start
CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start
CODE_SEG_64 equ gdt_code64 - gdt_start
DATA_SEG_64 equ gdt_data64 - gdt_start
[bits 32]

init_pm:
    mov ax, DATA_SEG
    mov ds, ax
    mov ss, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ebp, 0x90000
    mov esp, ebp
    mov esi, msg_pm
    mov edi, 0xB8000 + 160
    call print_string_pm
    mov edi, 0x1000
    xor eax, eax
    mov ecx, 4096 * 6 / 4 
    rep stosd
    mov edi, 0x1000
    mov cr3, edi
    mov DWORD [0x1000], 0x2003
    mov DWORD [0x1FF8], 0x2003
    mov DWORD [0x2000], 0x3003 
    mov DWORD [0x2008], 0x4003 
    mov DWORD [0x2010], 0x5003 
    mov DWORD [0x2018], 0x6003 
    mov DWORD [0x2FF0], 0x3003 
    mov DWORD [0x2FF8], 0x3003 
    mov edi, 0x3000
    mov eax, 0x00000083
    mov ecx, 512
.fill_pd0:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .fill_pd0
    mov edi, 0x4000
    mov ecx, 512
.fill_pd1:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .fill_pd1
    mov edi, 0x5000
    mov ecx, 512
.fill_pd2:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .fill_pd2
    mov edi, 0x6000
    mov ecx, 512
.fill_pd3:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .fill_pd3
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    jmp CODE_SEG_64:init_lm

print_string_pm:
    push eax
.loop:
    mov al, [esi]
    or al, al
    jz .done
    mov [edi], al
    mov byte [edi+1], 0x0F
    add esi, 1
    add edi, 2
    jmp .loop
.done:
    pop eax
    ret
msg_pm db "Protected Mode OK", 0
[bits 64]

init_lm:
    mov ax, DATA_SEG_64
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, 0x90000
    mov rsi, msg_lm
    mov rdi, 0xB8000 + 320
    call print_string_lm
    mov rsi, msg_lm_serial
    call print_serial_lm
    mov rsi, msg_pre_jump
    call print_serial_lm
    mov rax, 0xFFFFFFFF80010000
    jmp rax
    cli
    hlt
    jmp $

print_string_lm:
    push rax
.loop:
    mov al, [rsi]
    or al, al
    jz .done
    mov [rdi], al
    mov byte [rdi+1], 0x1B
    add rsi, 1
    add rdi, 2
    jmp .loop
.done:
    pop rax
    ret

print_serial_lm:
    push rax
    push rdx
.loop:
    mov al, [rsi]
    or al, al
    jz .done
    mov bl, al
.wait:
    mov dx, 0x3FD
    in al, dx
    test al, 0x20
    jz .wait
    mov dx, 0x3F8
    mov al, bl
    out dx, al
    inc rsi
    jmp .loop
.done:
    pop rdx
    pop rax
    ret
msg_lm           db "MOMMY managed to write VRAM <3", 0
msg_lm_serial    db "Long Mode Serial OK!", 13, 10, 0
msg_pre_jump     db "Jumping to Kernel...", 13, 10, 0
msg_kernel_ok    db "Kernel loaded.", 13, 10, 0
vbe_info_block: times 512 db 0
mode_info_block: times 256 db 0
msg_vesa_err db "VESA Error!", 13, 10, 0
