use spin::Mutex;
use lazy_static::lazy_static;

use x86_64::instructions::port::Port;

//https://wiki.osdev.org/Serial_Ports

pub const COM_1: u16 = 0x3F8;

pub struct Serial {
  port: Port<u8>,
  line_register_port: Port<u8>,
}

impl Serial {
  pub fn new(p: u16) -> Self {
    Self { port: Port::new(p), line_register_port: Port::new(p + 5) }
  }

  unsafe fn _write_char(&mut self, c: char) {
    //https://wiki.osdev.org/Serial_Ports#Line_Status_Register
    //wait until the "Transmitter holding register empty (THRE)" bit is set
    //THRE bit is set if data can be sent
    while (self.line_register_port.read() & 0x20) == 0 {}
    self.port.write(c as u8);
  }

  pub unsafe fn write_text(&mut self, text: &str) {
    for c in text.chars() {
      self._write_char(c);
    }
  }
}

lazy_static! {
  pub static ref SERIAL1: Mutex<Serial> = {
    let serial_port = Serial::new(COM_1);
    Mutex::new(serial_port)
  };
}

