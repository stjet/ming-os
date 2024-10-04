use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;

use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use blake2::{ Blake2b512, Digest };

const PASSWORD_HASH: [u8; 64] = [220, 88, 183, 188, 240, 27, 107, 181, 58, 191, 198, 170, 114, 38, 7, 148, 6, 179, 75, 128, 231, 171, 172, 220, 85, 38, 36, 113, 116, 146, 70, 197, 163, 179, 158, 192, 130, 53, 247, 48, 47, 209, 95, 96, 179, 211, 4, 122, 254, 127, 21, 165, 139, 199, 151, 226, 216, 176, 123, 41, 194, 221, 58, 69];

pub struct LockScreen {
  dimensions: Dimensions,
  input_password: String,
}

impl WindowLike for LockScreen {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘂' { //the enter key
          //check password
          let mut hasher = Blake2b512::new();
          hasher.update(self.input_password.as_bytes());
          if hasher.finalize() == PASSWORD_HASH.into() {
            WindowMessageResponse::Request(WindowManagerRequest::Unlock)
          } else {
            self.input_password = String::new();
            WindowMessageResponse::JustRerender
          }
        } else if key_press.key == '𐘁' { //backspace
          let p_len = self.input_password.len();
          if p_len != 0 {
            self.input_password = self.input_password[..p_len - 1].to_string();
          }
          WindowMessageResponse::JustRerender
        } else {
          self.input_password += &key_press.key.to_string();
          WindowMessageResponse::JustRerender
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, _theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![
      DrawInstructions::Rect([0, 0], self.dimensions, [0, 0, 0]),
      DrawInstructions::Text([4, 4], "times-new-roman", "The bulldozer outside the kitchen window was quite a big one.".to_string(), [255, 255, 255], [0, 0, 0]),
      DrawInstructions::Text([4, 4 + 16], "times-new-roman", "\"Yellow,\" he thought, and stomped off back to his bedroom to get dressed.".to_string(), [255, 255, 255], [0, 0, 0]),
      DrawInstructions::Text([4, 4 + 16 * 2], "times-new-roman", "He stared at it.".to_string(), [255, 255, 255], [0, 0, 0]),
      DrawInstructions::Text([4, 4 + 16 * 3], "times-new-roman", "Password: ".to_string(), [255, 255, 255], [0, 0, 0]),
      DrawInstructions::Text([77, 4 + 16 * 3], "times-new-roman", "*".repeat(self.input_password.len()), [255, 255, 255], [0, 0, 0]),
    ]
  }
  
  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::LockScreen
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    dimensions //fullscreen
  }
}

impl LockScreen {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
      input_password: String::new(),
    }
  }
}

