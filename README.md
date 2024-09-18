The project is an attempt to get something similar to [Mingde](https://github.com/stjet/mingde) as its own OS, while writing all the drawing and driver code. The secondary goal is to learn more about CPUs and memory and whatnot. But not too much.

Uses [Philipp Oppermann's Rust OS dev tutorial](https://os.phil-opp.com). In particular the files `allocator.rs`, `gdt.rs`, `memory.rs` and large parts of `interrupts.rs` are more or less completely copied.

It does use version 0.11 of the bootloader crate, which is newer than what the tutorial uses.

All the framebuffer stuff is original, as well as the window manager stuff. The mouse and keyboard drivers are not written yet, but will also be original.

