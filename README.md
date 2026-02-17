
# 🍼 Mommy OS - The OS That Loves You (Whether You Like It Or Not)

Welcome to the most affectionate operating system ever written in Rust. It watches you type, it judges your commands, and sometimes it even executes them!

## Features ✨

*   **Pings that actually wait for you:** We fixed the timeouts so you don't get ghosted by Google.
*   **A Filesystem that remembers:** Bitmap PMM ensures we don't accidentally overwrite your `ls` command with network packets (oops).
*   **MOM Executables:** Because `ELF` is for nerds. We run `.mom` binaries now.
*   **Zero Comments:** The code explains itself. Or it screams internally. One of the two.
*   **Unformatted Code:** The code is not formatted because I was lazy and cargo fmt made it explode.

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
- `ls` - Look at things.
- `ping google.com` - Talk to the outside world.
- `echo <messsage>` - Talk to yourself.
- `shutdown` - Rage quit.

---
*Made with love, panic handlers, and absolutely no memory leaks (anymore).*