use alloc::vec;
use alloc::vec::Vec;
use alloc::string::ToString;

use crate::components::Component;
use crate::framebuffer::{ get_font_max_height, Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::DrawInstructions;

pub struct HighlightButton<T> {
  name_: &'static str,
  top_left: Point,
  size: Dimensions,
  text: &'static str,
  pub highlighted: bool,
  click_return: T,
  toggle_highlight_return: T, //also unhighlight return
}

impl<T: Clone> Component<T> for HighlightButton<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    match message {
      WindowMessage::Focus | WindowMessage::Unfocus => {
        self.highlighted = !self.highlighted;
        Some(self.toggle_highlight_return.clone())
      },
      WindowMessage::FocusClick => {
        //we know this click was for this button, otherwise window wouldn't have given us this message
        Some(self.click_return.clone())
      },
      _ => None,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let font_height = get_font_max_height("times-new-roman").unwrap() as usize;
    if self.highlighted {
      vec![
        //highlight background
        DrawInstructions::Rect(self.top_left, self.size, theme_info.top),
        DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], "times-new-roman", self.text.to_string(), theme_info.text_top, theme_info.top),
      ]
    } else {
      vec![
        DrawInstructions::Rect(self.top_left, self.size, theme_info.background),
        DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], "times-new-roman", self.text.to_string(), theme_info.text, theme_info.background),
      ]
    }
  }

  //properties
  fn focusable(&self) -> bool {
    true
  }

  fn clickable(&self) -> bool {
    true
  }
  
  fn name(&self) -> &'static str {
    self.name_
  }
}

impl<T> HighlightButton<T> {
  pub fn new(name_: &'static str, top_left: Point, size: Dimensions, text: &'static str, click_return: T, toggle_highlight_return: T, highlighted: bool) -> Self {
    Self {
      name_,
      top_left,
      size,
      text,
      click_return,
      toggle_highlight_return,
      highlighted,
    }
  }
}

