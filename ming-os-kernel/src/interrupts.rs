use lazy_static::lazy_static;
use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode };
use x86_64::instructions::port::PortReadOnly;

use crate::gdt;
use crate::keyboard::scancode_to_char;
use crate::window_manager::keyboard_emit;
use crate::apic::lapic::LAPIC;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  Timer = PIC_1_OFFSET,
  Keyboard,
}

impl InterruptIndex {
  fn as_u8(self) -> u8 {
    self as u8
  }

  fn as_usize(self) -> usize {
    usize::from(self.as_u8())
  }
}

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    //
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    idt
  };
}

pub fn init_idt() {
  IDT.load();
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {

  panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
  //println!("Accessed Address: {:?}", Cr2::read());
  //println!("Error Code: {:?}", error_code);
  //println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT\n{:#?}\n{}", stack_frame, error_code);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
  unsafe {
    LAPIC.end_interrupt();
  }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
  let mut port = PortReadOnly::new(0x60);
  let scancode: u8 = unsafe { port.read() };

  //only handle presses, not releases
  let maybe_key_char = scancode_to_char(scancode);

  if let Some(key_char) = maybe_key_char {
    //send event to window manager
    keyboard_emit(key_char);
  }

  //

  unsafe {
    LAPIC.end_interrupt();
  }
}

