use x86_64::instructions::port::Port;

#[derive(Default, PartialEq)]
pub enum MousePacketState {
  #[default]
  First,
  Second,
  Third,
}

#[derive(Default)]
pub struct MousePacket {
  pub packet_state: MousePacketState,
  //discard if true
  pub overflowed: bool,
  //true is negative 
  pub y_sign: bool,
  pub x_sign: bool,
  //change in mouse coords
  pub x_delta: u8,
  pub y_delta: u8,
  //whether buttons are pressed
  pub middle: bool,
  pub right: bool,
  pub left: bool,
}

#[derive(Debug)]
pub struct MouseChange {
  pub x_delta: i16,
  pub y_delta: i16,
  //whether buttons are pressed
  pub middle: bool,
  pub right: bool,
  pub left: bool,
}

pub const MOUSE_DATA: u16 = 0x60; //same as keyboard

pub struct MouseSetup {
  port: Port<u8>,
  command_port: Port<u8>,
}

impl MouseSetup {
  pub fn new() -> Self {
    Self { port: Port::new(MOUSE_DATA), command_port: Port::new(MOUSE_DATA + 4) }
  }

  unsafe fn mouse_write(&mut self, write_command: bool, byte: u8) {
    //https://wiki.osdev.org/Mouse_Input#Waiting_to_Send_Bytes_to_Port_0x60_and_0x64
    while (self.command_port.read() & 0b0000_0010) != 0 {}
    if write_command {
      self.command_port.write(byte);
    } else {
      self.port.write(byte);
    }
  }

  unsafe fn mouse_read(&mut self) -> u8 {
    //https://wiki.osdev.org/Mouse_Input#Waiting_to_Send_Bytes_to_Port_0x60_and_0x64
    while (self.command_port.read() & 0b0000_0001) == 0 {}
    self.port.read()
  }

  pub unsafe fn init(&mut self) {
    //https://wiki.osdev.org/Mouse_Input#Aux_Input_Enable_Command
    self.mouse_write(true, 0xA8);
    //https://wiki.osdev.org/Mouse_Input#Set_Compaq_Status/Enable_IRQ12
    //https://wiki.osdev.org/%228042%22_PS/2_Controller (the Status Register part)
    //send the command byte 0x20 ("Get Compaq Status Byte") to the PS2 controller on port 0x64.
    self.mouse_write(true, 0x20);
    //The very next byte returned should be the Status byte.
    let mut status = self.mouse_read();
    //After you get the Status byte, you need to set bit number 1 (value=2, Enable IRQ12), and clear bit number 5 (value=0x20, Disable Mouse Clock).
    status |= 0b0000_0010;
    status &= !0b0010_0000;
    //Then send command byte 0x60 ("Set Compaq Status") to port 0x64, followed by the modified Status byte to port 0x60
    self.mouse_write(true, 0x60);
    self.mouse_write(false, status);
    //0xF6 sets defaults, 0xF4 enables streaming
    self.mouse_write(true, 0xD4);
    self.mouse_write(false, 0xF6);
    self.mouse_read();
    self.mouse_write(true, 0xD4);
    self.mouse_write(false, 0xF4);
    self.mouse_read();
  }
}

pub fn mouse_init() {
  let mut mouse_setup = MouseSetup::new();
  unsafe { mouse_setup.init(); }
}

