use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, WindowMessage };
use crate::framebuffer::Dimensions;
use alloc::vec;
use alloc::vec::Vec;

pub struct DesktopBackground {
  dimensions: Dimensions,
}

impl WindowLike for DesktopBackground {
  fn handle_message(&mut self, message: WindowMessage) -> bool {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        true
      },
      //_ => {},
    };
    false
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::DesktopBackground
  }

  //simple
  fn draw(&self) -> Vec<DrawInstructions> {
    vec![DrawInstructions::Rect([0, 0], self.dimensions, [255, 0, 0])]
  }
}

impl DesktopBackground {
  pub fn new() -> Self {
    Self { dimensions: [0, 0] }
  }
}

