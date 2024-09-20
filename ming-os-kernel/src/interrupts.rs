use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame };
use x86_64::instructions::port::PortReadOnly;

use crate::gdt;
use crate::keyboard::scancode_to_char;
use crate::window_manager::{ keyboard_emit, mouse_emit };
use crate::mouse::{ MousePacket, MousePacketState, MouseChange };

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  Timer = PIC_1_OFFSET,
  Keyboard,
  Mouse = PIC_2_OFFSET + 4,
}

impl InterruptIndex {
  fn as_u8(self) -> u8 {
    self as u8
  }

  fn as_usize(self) -> usize {
    usize::from(self.as_u8())
  }
}

pub static PICS: spin::Mutex<ChainedPics> =
  spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
    idt
  };
}

pub fn init_idt() {
  IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
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
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  }
}

lazy_static! {
  static ref mouse_packet: spin::Mutex<MousePacket> = spin::Mutex::new(Default::default());
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
  let mut port = PortReadOnly::new(0x60);

  //https://wiki.osdev.org/Mouse_Input#Mouse_Packet_Info
  /*
  3 bytes:
  - Byte 1: y overflow, x overflow, y sign bit, x sign bit, always 1, middle button, right button, left button
  - Byte 2: x movement
  - Byte 3: y movement
  */
  let packet: u8 = unsafe { port.read() };

  let mut locked = mouse_packet.lock();

  if locked.packet_state == MousePacketState::First && (packet & 0b0000_1000 != 0b0000_1000) {
    //does nothing; makes sure alignment is right. that bit should always be 1
  } else if locked.packet_state == MousePacketState::First {
    locked.overflowed = packet & 0b1000_0000 == 0b1000_0000 || packet & 0b0100_0000 == 0b0100_0000;
    locked.y_sign = packet & 0b0010_0000 == 0b0010_0000;
    locked.x_sign = packet & 0b0001_0000 == 0b0001_0000;
    locked.middle = packet & 0b0000_0100 == 0b0000_0100;
    locked.right = packet & 0b0000_0010 == 0b0000_0010;
    locked.left = packet & 0b0000_0001 == 0b0000_0001;
    locked.packet_state = MousePacketState::Second;
  } else if locked.packet_state == MousePacketState::Second {
    locked.x_delta = packet;
    locked.packet_state = MousePacketState::Third;
  } else if locked.packet_state == MousePacketState::Third {
    locked.y_delta = packet;
    locked.packet_state = MousePacketState::First;
    //send event to window manager
    if !locked.overflowed {
      //I think all these random packets with delta 255 are some errors, so cap them
      if locked.x_delta > 100 {
        locked.x_delta = 8;
      }
      if locked.y_delta > 100 {
        locked.y_delta = 8;
      }
      mouse_emit(MouseChange {
        x_delta: if locked.x_sign { -(locked.x_delta as i16) } else { locked.x_delta as i16 },
        //mouse says negative is down. we think negative y is up (0, 0) is top left
        y_delta: if locked.y_sign { locked.y_delta as i16 } else { -(locked.y_delta as i16) },
        middle: locked.middle,
        right: locked.right,
        left: locked.left,
      });
    }
    //
  }

  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
  }
}

