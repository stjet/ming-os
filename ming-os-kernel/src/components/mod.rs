use alloc::vec::Vec;

use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::DrawInstructions;
use crate::framebuffer::Point;

pub mod button;

pub trait Component<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T>;
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;
  fn point_inside(&self, point: Point) -> bool;

  //properties
  fn clickable(&self) -> bool;
  fn name(&self) -> &'static str; //should be unique
}

