use alloc::vec;
use alloc::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

pub struct StartMenu {
  dimensions: Dimensions,
}

impl WindowLike for StartMenu {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }
  
  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::StartMenu
  }
  
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![
      DrawInstructions::Rect([0, 0], self.dimensions, theme_info.border_left_top),
    ]
  }
  //
}

impl StartMenu {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
    }
  }
}

