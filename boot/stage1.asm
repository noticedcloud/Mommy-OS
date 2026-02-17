[bits 16]
[org 0x7c00]

start:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00
    mov [boot_drive], dl
    mov dx, 0x3F8 + 1
    xor al, al
    out dx, al
    mov dx, 0x3F8 + 3
    mov al, 0x80
    out dx, al
    mov dx, 0x3F8
    mov al, 0x03
    out dx, al
    mov dx, 0x3F8 + 1
    xor al, al
    out dx, al
    mov dx, 0x3F8 + 3
    mov al, 0x03
    out dx, al
    mov dx, 0x3F8 + 2
    mov al, 0xC7
    out dx, al
    mov dx, 0x3F8 + 4
    mov al, 0x0B
    out dx, al
    mov dx, 0x3F8 + 4
    mov al, 0x0F
    out dx, al
    mov si, msg
    call print_serial
    call print_vga
    mov ah, 0
    mov dl, [boot_drive]
    int 0x13
    jc disk_error
    mov bx, 0x7E00
    mov ah, 0x02
    mov al, 4
    mov ch, 0
    mov dh, 0
    mov cl, 2
    mov dl, [boot_drive]
    int 0x13
    jc disk_error
    mov si, msg_stage2
    call print_vga
    mov dl, [boot_drive]
    jmp 0x0000:0x7E00

disk_error:
    mov si, msg_err
    call print_vga
    cli
    hlt
    cli
    hlt
    jmp $

print_serial:
    lodsb
    or al, al
    jz .done
    mov dx, 0x3F8 + 5
.wait_transmit:
    in al, dx
    test al, 0x20
    jz .wait_transmit
    mov dx, 0x3F8
    mov al, [si-1]
    out dx, al
    jmp print_serial
.done:
    ret

print_vga:
    lodsb
    or al, al
    jz .done_vga
    mov ah, 0x0e
    mov bh, 0
    mov bl, 0x07
    int 0x10
    jmp print_vga
.done_vga:
    ret

msg:
    db 13, 10
    db "Welcome to MOMMY OS! <3 Mommy is watching.", 13, 10
    db "Serial active at 38400 baud! Now mommy sees everything...", 13, 10, 0

msg_stage2:
    db "Stage 2 ready, let's go...", 13, 10, 0

msg_err:
    db "Disk Error!", 13, 10, 0

boot_drive:
    db 0
times 510 - ($ - $$) db 0
dw 0xaa55
