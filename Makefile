ASM = nasm
RUST_KERNEL_DIR = kernel
BOOT_DIR = boot
IMG = $(BOOT_DIR)/myos.img
ISO = $(BOOT_DIR)/myos.iso

STAGE2_SECTOR = 1
KERNEL_SECTOR = 20

all: img

$(BOOT_DIR)/stage1.bin: $(BOOT_DIR)/stage1.asm
	$(ASM) -f bin $< -o $@

$(BOOT_DIR)/stage2.bin: $(BOOT_DIR)/stage2.asm
	$(ASM) -f bin $< -o $@

RS_SOURCES := $(shell find $(RUST_KERNEL_DIR)/src -name '*.rs')

userspace:
	make -C userspace

$(BOOT_DIR)/kernel.bin: $(RS_SOURCES) $(RUST_KERNEL_DIR)/linker.ld userspace
	cd $(RUST_KERNEL_DIR) && cargo build --target x86_64-unknown-none --release
	cd $(RUST_KERNEL_DIR) && cargo objcopy --release -- -O binary ../$@

img: $(BOOT_DIR)/stage1.bin $(BOOT_DIR)/stage2.bin $(BOOT_DIR)/kernel.bin
	@dd if=/dev/zero of=$(IMG) bs=512 count=2880 status=none
	@dd if=$(BOOT_DIR)/stage1.bin of=$(IMG) conv=notrunc status=none
	@dd if=$(BOOT_DIR)/stage2.bin of=$(IMG) seek=$(STAGE2_SECTOR) conv=notrunc status=none
	@dd if=$(BOOT_DIR)/kernel.bin of=$(IMG) seek=$(KERNEL_SECTOR) conv=notrunc status=none

iso: img
	@if command -v xorriso >/dev/null 2>&1; then \
		xorriso -as mkisofs -b myos.img -hide myos.img -o $(ISO) $(BOOT_DIR); \
	elif command -v mkisofs >/dev/null 2>&1; then \
		mkisofs -b myos.img -hide myos.img -o $(ISO) $(BOOT_DIR); \
	elif command -v genisoimage >/dev/null 2>&1; then \
		genisoimage -b myos.img -hide myos.img -o $(ISO) $(BOOT_DIR); \
	else \
	else \
		exit 1; \
	fi


run: img
	-pkill -f qemu-system-x86_64
	qemu-system-x86_64 -drive format=raw,file=$(IMG),if=floppy -serial mon:stdio -no-reboot

clean:
	rm -f $(BOOT_DIR)/*.bin $(IMG) $(ISO)
	cd $(RUST_KERNEL_DIR) && cargo clean
	make -C userspace clean

.PHONY: all img iso run clean userspace
