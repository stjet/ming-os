use alloc::vec;
use alloc::vec::Vec;

use crate::components::Component;
use crate::framebuffer::{ get_font_max_height, Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::DrawInstructions;

//we need a text width and height measure function first
pub enum ButtonAlignment {
  Centre,
  Left,
  Right,
}

pub struct Button<T> {
  name_: &'static str,
  top_left: Point,
  size: Dimensions,
  text: &'static str,
  draw_bg: bool,
  pub inverted: bool,
  alignment: ButtonAlignment,
  click_return: T,
}

impl<T: Clone> Component<T> for Button<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    match message {
      WindowMessage::MouseLeftClick(_) => {
        //we know this left click was for this button, otherwise window wouldn't have given us this message
        self.inverted = !self.inverted;
        Some(self.click_return.clone())
      },
      _ => None,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    //to make sure the text gets vertically centred
    let font_height = get_font_max_height("times-new-roman").unwrap() as usize;
    vec![
      //top and left border
      DrawInstructions::Rect(self.top_left, [self.size[0], 2], if self.inverted { theme_info.border_right_bottom } else { theme_info.border_left_top }),
      DrawInstructions::Rect(self.top_left, [2, self.size[1]], if self.inverted { theme_info.border_right_bottom } else { theme_info.border_left_top }),
      //right and bottom border
      DrawInstructions::Rect([self.top_left[0] + self.size[0] - 2, self.top_left[1]], [2, self.size[1]], if self.inverted { theme_info.border_left_top } else { theme_info.border_right_bottom }),
      DrawInstructions::Rect([self.top_left[0], self.top_left[1] + self.size[1] - 2], [self.size[0], 2], if self.inverted { theme_info.border_left_top } else { theme_info.border_right_bottom }),
      //the background if self.draw_bg
      //DrawInstructions::Rect(),
      //the text (for now, hardcoded top left)
      DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], "times-new-roman", "Start", theme_info.text, theme_info.background),
    ]
  }

  fn point_inside(&self, point: Point) -> bool {
    let bottom_right = [self.top_left[0] + self.size[0], self.top_left[1] + self.size[1]];
    return point[0] >= self.top_left[0] && point[0] <= bottom_right[0] && point[1] >= self.top_left[1] && point[1] <= bottom_right[1];
  }

  //properties
  fn clickable(&self) -> bool {
    true
  }
  
  fn name(&self) -> &'static str {
    self.name_
  }
}

impl<T> Button<T> {
  pub fn new(name_: &'static str, top_left: Point, size: Dimensions, text: &'static str, click_return: T, draw_bg: bool, alignment: Option<ButtonAlignment>) -> Self {
    Self {
      name_,
      top_left,
      size,
      text,
      click_return,
      draw_bg,
      inverted: false,
      alignment: alignment.unwrap_or(ButtonAlignment::Centre),
    }
  }
}

