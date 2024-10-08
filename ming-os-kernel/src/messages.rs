use alloc::boxed::Box;
use alloc::fmt;
use alloc::vec::Vec;

use crate::keyboard::KeyChar;
use crate::framebuffer::Dimensions;
use crate::window_manager::WindowLike;

pub enum WindowManagerMessage {
  KeyChar(KeyChar),
  //
}

type WindowBox = Box<dyn WindowLike + Send>;

impl PartialEq for WindowBox {
  fn eq(&self, _other: &Self) -> bool {
    //lol
    true
  }
}


#[derive(PartialEq)]
pub enum WindowManagerRequest {
  OpenWindow(WindowBox),
  CloseStartMenu,
  Unlock,
  Lock,
  //
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

pub struct KeyPress {
  pub key: char,
  pub held_special_keys: Vec<&'static str>,
  //
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
  Left,
  Down,
  Up,
  Right,
}

//todo, rename to CommandType
#[derive(PartialEq)]
pub enum ShortcutType {
  StartMenu,
  SwitchWorkspace(u8),
  MoveWindowToWorkspace(u8),
  FocusNextWindow,
  QuitWindow,
  MoveWindow(Direction),
  MoveWindowToEdge(Direction),
  //
}

pub type WindowsVec = Vec<(usize, &'static str)>;

pub enum InfoType {
  //let taskbar know what the current windows in the workspace are
  WindowsInWorkspace(WindowsVec, usize), //Vec<title, name)>, focused id
  //
}

pub enum WindowMessage {
  Init(Dimensions),
  KeyPress(KeyPress),
  Shortcut(ShortcutType),
  Info(InfoType),
  Focus,
  Unfocus,
  FocusClick,
  //
}
