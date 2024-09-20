#![no_std]
#![no_main]

#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use core::mem::replace;
use alloc::format;

use bootloader_api::{ entry_point, BootInfo, BootloaderConfig };
use bootloader_api::config::Mapping;
use bootloader_api::info::{ FrameBuffer, Optional };

use x86_64::VirtAddr;

mod memory;
use memory::BootInfoFrameAllocator;

mod allocator;

mod gdt;

mod interrupts;

mod framebuffer;

mod window_manager;
use window_manager::init;

mod window_likes;

mod themes;

mod keyboard;

mod serial;
use serial::SERIAL1;

mod mouse;
use mouse::mouse_init;

mod messages;

pub fn hlt_loop() -> ! {
  loop {
    x86_64::instructions::hlt();
  }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
  unsafe { SERIAL1.lock().write_text(&format!("{}", panic_info)); }
  hlt_loop();
}

static BOOTLOADER_CONFIG: BootloaderConfig = {
  let mut config = BootloaderConfig::new_default();
  config.mappings.physical_memory = Some(Mapping::Dynamic);
  config
};

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
  //double fault interrupts
  gdt::init();
  interrupts::init_idt();
  //mouse needs to be initalised BEFORE the interrupts come
  //obvious in hindsight
  mouse_init();
  //hardware interrupts
  unsafe { interrupts::PICS.lock().initialize() };
  x86_64::instructions::interrupts::enable();
  //memory
  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe {
    BootInfoFrameAllocator::init(&boot_info.memory_regions)
  };
  allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("heap initialization failed");

  //framebuffer
  let framebuffer = replace(&mut boot_info.framebuffer, Optional::None);
  let framebuffer: FrameBuffer = framebuffer.into_option().unwrap();

  //set up wm and whatnot
  init(framebuffer);

  hlt_loop();
}

entry_point!(kernel_main, config=&BOOTLOADER_CONFIG);

