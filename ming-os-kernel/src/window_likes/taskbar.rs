use alloc::vec;
use alloc::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::WindowMessage;
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

pub struct Taskbar {
  dimensions: Dimensions,
}

impl WindowLike for Taskbar {
  fn handle_message(&mut self, message: WindowMessage) -> bool {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        true
      },
      _ => false,
    }
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Taskbar
  }

  //simple
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![
      //top thin white border
      DrawInstructions::Rect([0, 0], [self.dimensions[0], 2], theme_info.border_left_top),
      //the actual taskbar background
      DrawInstructions::Rect([0, 2], [self.dimensions[0], self.dimensions[1] - 2], theme_info.background),
    ]
  }
}

impl Taskbar {
  pub fn new() -> Self {
    Self { dimensions: [0, 0] }
  }
}


