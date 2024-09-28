Primarily, the goal of this project is to implement [mingde](https://github.com/stjet/mingde) (in looks, function, and code architecture) as an operating system running on bare metal on a real machine (my laptop). Why? Because I think it would be cool and fun to do. And to some extent, it would be nice to learn something about memory and CPUs and whatnot. But not too much.

Also, it will be keyboard only. I wrote a (not very good) mouse driver and made the mouse in the GUI and whatnot, but decided to remove it after seeing how much more pain rerendering was going to be. The mouse is really not needed or used that often anyways, and I can program in whatever keyboard ~~shortcuts~~ commands needed.

This is really more of a program running on bare metal than an operating system, because there is no userspace and it's all one big executable.

Because I haven't got the foggiest clue how to write a USB driver, files will be loaded through the bootloader's ramdisk (`fs.json`), and then exfiltrated with QR codes. At least, that's the current plan. Yeah, it's pretty dumb, but there's no serial port on my laptop.

Initially, I followed [Philipp Oppermann's Rust OS dev tutorial](https://os.phil-opp.com). In particular the files `allocator.rs`, `gdt.rs`, `memory.rs` and `interrupts.rs` are more or less completely copied. I made some simple changes to use version 0.11 of the `bootloader` crate. Then, the `apic` stuff was lifted from [xuanplus/rust-os-dev](https://github.com/xuanplus/rust-os-dev) with some changes so it actually works on my machine (init lapic before ioapic, xuan!).

