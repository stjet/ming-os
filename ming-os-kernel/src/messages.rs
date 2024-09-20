use crate::mouse::MouseChange;
use crate::keyboard::KeyChar;
use crate::framebuffer::{ Dimensions, Point };

pub enum WindowManagerMessage {
  KeyChar(KeyChar),
  MouseChange(MouseChange),
  //
}

pub struct KeyboardPress {
  pub key: char,
  //
}

pub struct MouseMove {
  pub coords: Point,
  pub left: bool, //whether left mouse button is down
  //
}

pub struct MouseLeftClick {
  pub coords: Point,
}

pub enum WindowMessage {
  Init(Dimensions),
  KeyboardPress(KeyboardPress),
  MouseMove(MouseMove),
  MouseMoveOutside,
  MouseLeftClick(MouseLeftClick),
  //
}
