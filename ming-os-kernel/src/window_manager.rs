use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;

use spin;
use bootloader_api::info::{ FrameBuffer, PixelFormat };
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;

use crate::framebuffer::{ FrameBufferWriter, Point, Dimensions, RGBColor };
use crate::window_likes::desktop_background::DesktopBackground;
use crate::window_likes::taskbar::Taskbar;
use crate::themes::{ ThemeInfo, Themes, get_theme_info };
use crate::mouse::MouseChange;
use crate::keyboard::KeyChar;
use crate::SERIAL1;
use crate::messages::*;

pub fn point_in_window(point: Point, top_left: Point, dimensions: Dimensions) -> bool {
  let bottom_right = [top_left[0] + dimensions[0], top_left[1] + dimensions[1]];
  return point[0] >= top_left[0] && point[0] <= bottom_right[0] && point[1] >= top_left[1] && point[1] <= bottom_right[1];
}

lazy_static! {
  static ref WRITER: spin::Mutex<FrameBufferWriter> = spin::Mutex::new(Default::default());
  static ref WM: spin::Mutex<WindowManager> = spin::Mutex::new(Default::default());
}

pub fn init(framebuffer: FrameBuffer) {
  let framebuffer_info = framebuffer.info();
  WRITER.lock().new(framebuffer_info, framebuffer.into_buffer());
  
  //WRITER.lock().draw_rect([0, 0], [5, 5], [0, 255, 255]);

  const TASKBAR_HEIGHT: usize = 38;

  WM.lock().init([framebuffer_info.width, framebuffer_info.height]);

  WM.lock().add_window_like(Box::new(DesktopBackground::new()), [0, 0], [framebuffer_info.width, framebuffer_info.height - TASKBAR_HEIGHT]);
  
  WM.lock().add_window_like(Box::new(Taskbar::new()), [0, framebuffer_info.height - TASKBAR_HEIGHT], [framebuffer_info.width, TASKBAR_HEIGHT]);

  without_interrupts(|| {
    WM.lock().render();
  });

  //
}

pub fn keyboard_emit(key_char: KeyChar) {
  unsafe { SERIAL1.lock().write_text(&format!("{:?}", &key_char)); }
  WM.lock().handle_message(WindowManagerMessage::KeyChar(key_char));
}

pub fn mouse_emit(mouse_change: MouseChange) {
  unsafe { SERIAL1.lock().write_text(&format!("{:?}", &mouse_change)); }
  WM.lock().handle_message(WindowManagerMessage::MouseChange(mouse_change));
}

pub enum DrawInstructions {
  Rect(Point, Dimensions, RGBColor),
}

#[derive(PartialEq)]
pub enum WindowLikeType {
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
}

pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> bool;

  //properties
  fn subtype(&self) -> WindowLikeType;
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;
}

pub struct WindowLikeInfo {
  id: usize,
  window_like: Box<dyn WindowLike + Send>,
  top_left: Point,
  dimensions: Dimensions,
}

#[derive(Default)]
pub struct WindowManager {
  id_count: usize,
  window_infos: Vec<WindowLikeInfo>,
  dimensions: Dimensions,
  theme: Themes,
  focused_window_id: Option<usize>,
  mouse_coords: Point,
}

pub const MOUSE_SIZE: [usize; 2] = [5, 5];

impl WindowManager {
  pub fn init(&mut self, dimensions: Dimensions) {
    self.dimensions = dimensions;
  }

  pub fn add_window_like(&mut self, mut window_like: Box<dyn WindowLike + Send>, top_left: Point, dimensions: Dimensions) {
    self.id_count = self.id_count + 1;
    let id = self.id_count;
    window_like.handle_message(WindowMessage::Init(dimensions));
    self.window_infos.push(WindowLikeInfo {
      id,
      window_like,
      top_left,
      dimensions,
    });
  }

  //

  pub fn handle_message(&mut self, message: WindowManagerMessage) {
    let rerender: bool = match message {
      WindowManagerMessage::KeyChar(key_char) => {
        //check if is special key (key releases are guaranteed to be special keys)
        //eg: ctrl, alt, command/windows, shift, or caps lock
        match key_char {
          KeyChar::Press(char) => {
            //
          },
          KeyChar::SpecialPress(special_key) => {
            //add to pressed keys
            //
          },
          KeyChar::SpecialRelease(special_key) => {
            //remove it from pressed keys
            //
          },
        };
        //
        false
      },
      WindowManagerMessage::MouseChange(mouse_change) => {
        if mouse_change.x_delta != 0 || mouse_change.y_delta != 0 {
          //handle mouse move
          self.mouse_coords = [
            (self.mouse_coords[0] as i16 + mouse_change.x_delta) as usize,
            (self.mouse_coords[1] as i16 + mouse_change.y_delta) as usize,
          ];
          //prevent mouse going off screen
          if self.mouse_coords[0] < 0 {
            self.mouse_coords[0] = 0;
          }
          if self.mouse_coords[1] < 0 {
            self.mouse_coords[1] = 0;
          }
          if self.mouse_coords[0] >= (self.dimensions[0] - MOUSE_SIZE[0]) {
            self.mouse_coords[0] = self.dimensions[0] - MOUSE_SIZE[0];
          }
          if self.mouse_coords[1] >= (self.dimensions[1] - MOUSE_SIZE[1]) {
            self.mouse_coords[1] = self.dimensions[1] - MOUSE_SIZE[1];
          }
          //send mouse move or mouse move outside to focused window
          if let Some(focused_window_id) = self.focused_window_id {
            let focused_window = &mut self.window_infos[focused_window_id];
            if point_in_window(self.mouse_coords, focused_window.top_left, focused_window.dimensions) {
              focused_window.window_like.handle_message(WindowMessage::MouseMove(MouseMove {
                coords: [
                  self.mouse_coords[0] + focused_window.top_left[0],
                  self.mouse_coords[1] + focused_window.top_left[1],
                ],
                left: mouse_change.left,
              }));
            } else {
              focused_window.window_like.handle_message(WindowMessage::MouseMoveOutside);
            }
          }
          //because we have to redraw the mouse
          true
        } else {
          //handle mouse click
          if mouse_change.left {
            //get window like that was clicked on.
            //it will return something, guaranteed because there is always a taskbar and desktop background
            let mut new_focused_index = self.window_infos.iter().rposition(|w| point_in_window(self.mouse_coords, w.top_left, w.dimensions)).unwrap();
            let changed_focus = Some(new_focused_index) == self.focused_window_id;
            self.focused_window_id = Some(new_focused_index);
            let focused_subtype = &self.window_infos[new_focused_index].window_like.subtype();
            if focused_subtype == &WindowLikeType::Window || focused_subtype == &WindowLikeType::StartMenu {
              //remove and push to back so it renders on top
              let removed = self.window_infos.remove(new_focused_index);
              self.window_infos.push(removed);
              new_focused_index = self.window_infos.len() - 1;
            }
            let focused_window = &mut self.window_infos[new_focused_index];
            let focus_resp = focused_window.window_like.handle_message(WindowMessage::MouseLeftClick(MouseLeftClick {
              coords: [
                self.mouse_coords[0] + focused_window.top_left[0],
                self.mouse_coords[1] + focused_window.top_left[1],
              ],
            }));
            changed_focus || focus_resp
          } else {
            //
            false
          }
        }
      },
      //
      _ => false,
    };
    if rerender {
      self.render();
    }
  }

  pub fn render(&mut self) {
    for window_info in &self.window_infos {
      //draw window decorations and what not
      //
      let theme_info = get_theme_info(&self.theme).unwrap();
      for instruction in window_info.window_like.draw(&theme_info) {
        match instruction {
          DrawInstructions::Rect(top_left, dimensions, color) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            //try and prevent overflows because WRITER will not
            let true_dimensions = [
              if dimensions[0] > (window_info.dimensions[0] - top_left[0]) { window_info.dimensions[0] } else { dimensions[0] },
              if dimensions[1] > (window_info.dimensions[1] - top_left[1]) { window_info.dimensions[1] } else { dimensions[1] },
            ];
            WRITER.lock().draw_rect(true_top_left, true_dimensions, color);
          },
          //_ => {},
        }
      }
    }
    //draw mouse pointer
    //must be MOUSE_SIZE
    WRITER.lock().draw_char([self.mouse_coords[0], self.mouse_coords[1]], "_icons", '0', [0, 255, 0], [0, 0, 0], None);
  }
}

