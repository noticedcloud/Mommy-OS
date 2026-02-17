PACKER=target/release/packer

all: packer_tool hello ls clear echo reboot shutdown linux_test

packer_tool:
	cd packer && cargo build --release

hello: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin hello --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/hello hello.bin
	$(PACKER) hello.bin hello.mom

ls: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin ls --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/ls ls.bin
	$(PACKER) ls.bin ls.mom

clear: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin clear --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/clear clear.bin
	$(PACKER) clear.bin clear.mom

echo: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin echo --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/echo echo.bin
	$(PACKER) echo.bin echo.mom

reboot: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin reboot --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/reboot reboot.bin
	$(PACKER) reboot.bin reboot.mom

shutdown: packer_tool
	RUSTFLAGS="-C link-arg=-Tlinker.ld -C relocation-model=pic" cargo build --release --bin shutdown --target x86_64-unknown-none
	objcopy -O binary target/x86_64-unknown-none/release/shutdown shutdown.bin
	$(PACKER) shutdown.bin shutdown.mom

linux_test:
	nasm -f elf64 linux_test.asm -o linux_test.o
	ld linux_test.o -o linux_test

clean:
	cargo clean
	cd packer && cargo clean
	rm -f *.mom *.bin *.o linux_test

