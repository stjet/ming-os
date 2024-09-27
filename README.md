Primarily, the goal of this project is to implement [mingde](https://github.com/stjet/mingde) (in looks, function, and code architecture) as an operating system running on bare metal on a real machine (my laptop). Why? Because I think it would be cool and fun to do. And to some extent, it would be nice to learn something about memory and CPUs and whatnot. But not too much.

This is really more of a program running on bare metal than an operating system, because there is no userspace and it's all one big executable.

The current hope for the project is to have a file system and file editing, so it can be used practically for writing (eg, notes or programs).

Ming-os initially supported mice, but after considering the performance implications (damn mouse moving means redrawing a lot of stuff), the project is now strictly keyboard-only. [This commit](https://github.com/stjet/ming-os/commit/0243b14d2b873f19d3ce8f9a0489e45ea4d537ff) is the last version of the OS with mouse support. It is slightly broken but can be fixed easily.

Initially, I followed [Philipp Oppermann's Rust OS dev tutorial](https://os.phil-opp.com). In particular the files `allocator.rs`, `gdt.rs`, `memory.rs` and `interrupts.rs` are more or less completely copied. I made some simple changes to use version 0.11 of the `bootloader` crate. Then, the `apic` stuff was lifted from [xuanplus/rust-os-dev](https://github.com/xuanplus/rust-os-dev) with some changes so it actually works on my machine (init lapic before ioapic, xuan!).

