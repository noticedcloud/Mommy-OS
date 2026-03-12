
# 🍼 Mommy OS - The OS That Loves You (Whether You Like It Or Not)

Welcome to the most affectionate operating system ever written in Rust. It watches you type, it judges your commands, and sometimes it even executes them!

## Features ✨

*   **Memory Management (The Mommy Way):** We divide RAM into **Kernel**, **Cradle** (safe space), and **Playpen** (where user code can make a mess). If the Playpen gets too full, it might "invade" the Cradle. Yes, we implemented geopolitical conflict in RAM.
*   **Mommy's Wardrobe (MSW):** A custom 64-bit filesystem with 4KB blocks. It features **Full-Disk Encryption (AES-256)** via Argon2id because Mommy doesn't want you looking at her secrets.
*   **Networking Stack from Scratch:** Written in pure Rust. We talk to Intel e1000 NICs directly. We have custom ARP, IPv4, UDP, DNS, and ICMP. Our `ping` even calculates latency because we care about how long you've been waiting.
*   **MOM Executables:** Because `ELF` is for nerds. We run `.mom` binaries (raw position-independent code) with a custom `MOM!` header.
*   **Zero Comments Policy:** Because if the code was hard to write, it should be hard to read. Mommy doesn't like the one's who steal code.
*   **Safety First (Mostly):** Built with `#![no_std]` and `#![no_main]`. We handle our own Panics, Page Faults, and General Protection Faults with style.

## Requirements 🛠️

You need these (unless you like compilation errors):
*   **Rust (Nightly):** Because stable is for weaklings.
*   **QEMU (qemu-system-x86_64):** To emulate the suffering.
*   **Make:** To execute `make run` without thinking.

## How to Run 🚀

Just type this and pray:

```bash
make run
```
in another shell type

```bash
vncviewer localhost:5900
```

Then talk to Mommy:
- `ls` - Look at things, may work, may crash the system, its a gamble like we all like.
- `ping google.com` - Talk to the outside world, may work, may not, who knows.
- `echo <messsage>` - Talk to yourself, may work, may crash the system, this is a gamble too.
- `shutdown` - Rage quit.

---
*Made with love, panic handlers, and absolutely no memory leaks (anymore... I HOPE).*
*Warning, if you think mommy is watching you, its true, be careful, not to be abducted by her, or maybe do it... i dont know you do you, if she abducts me im happy fr... yeah i should stop this shit*