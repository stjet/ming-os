#![no_std]
#![no_main]

#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::mem::replace;

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

pub fn hlt_loop() -> ! {
  loop {
    x86_64::instructions::hlt();
  }
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
  init(framebuffer);
  /*without_interrupts(|| {
  //
  WRITER.lock().draw_pixel([0, 0], [255, 0, 0]);
  WRITER.lock().draw_rect([1, 1], [300, 100], [0, 255, 0]);
  WRITER.lock().draw_text([10, 20], "times-new-roman", "abcdefghijklmnopqrstuvwxyz", [255, 255, 255], [0, 255, 0], 0);
  WRITER.lock().draw_text([10, 37], "times-new-roman", "0123456789.():{},", [255, 255, 255], [0, 255, 0], 0);
  WRITER.lock().draw_text([10, 54], "times-new-roman", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", [255, 255, 255], [0, 255, 0], 0);
  });*/
  //

  hlt_loop();
}

entry_point!(kernel_main, config=&BOOTLOADER_CONFIG);

