use alloc::boxed::Box;
use alloc::fmt;

use crate::mouse::MouseChange;
use crate::keyboard::KeyChar;
use crate::framebuffer::{ Dimensions, Point };
use crate::window_manager::WindowLike;

pub enum WindowManagerMessage {
  KeyChar(KeyChar),
  MouseChange(MouseChange),
  //
}

pub enum WindowManagerRequest {
  OpenWindow(Box<dyn WindowLike + Send>, Point, Dimensions),
  //
}

impl PartialEq for WindowManagerRequest {
  fn eq(&self, _other: &Self) -> bool {
    //lol
    true
  }
}

impl fmt::Debug for WindowManagerRequest{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WindowManagerRequest lmao")
  }
}

#[derive(PartialEq, Debug)]
pub enum WindowMessageResponse {
  Request(WindowManagerRequest),
  JustRerender,
  DoNothing,
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

#[derive(PartialEq)]
pub enum ShortcutType {
  StartMenu,
  //
}

pub enum WindowMessage {
  Init(Dimensions),
  KeyboardPress(KeyboardPress),
  MouseMove(MouseMove),
  MouseMoveOutside,
  MouseLeftClick(MouseLeftClick),
  Shortcut(ShortcutType),
  //
}
