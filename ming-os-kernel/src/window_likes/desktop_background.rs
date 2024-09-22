use alloc::vec;
use alloc::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

pub struct DesktopBackground {
  dimensions: Dimensions,
}

impl WindowLike for DesktopBackground {
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
    WindowLikeType::DesktopBackground
  }

  //simple
  fn draw(&self, _theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![DrawInstructions::Rect([0, 0], self.dimensions, [0, 128, 128])]
  }
}

impl DesktopBackground {
  pub fn new() -> Self {
    Self { dimensions: [0, 0] }
  }
}

