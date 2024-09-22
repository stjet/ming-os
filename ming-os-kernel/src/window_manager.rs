use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::fmt;
use alloc::boxed::Box;
use alloc::format;

use spin;
use bootloader_api::info::FrameBuffer;
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

pub const TASKBAR_HEIGHT: usize = 38;

//also point in button
pub fn point_in_window(point: Point, top_left: Point, dimensions: Dimensions) -> bool {
  let bottom_right = [top_left[0] + dimensions[0], top_left[1] + dimensions[1]];
  return point[0] >= top_left[0] && point[0] <= bottom_right[0] && point[1] >= top_left[1] && point[1] <= bottom_right[1];
}

/*
pub fn rect_in_window(rect_top_left: Point, rect_dimensions: Dimensions, top_left: Point, dimensions: Dimensions) -> bool {
  //see if any of the 4 corners are in rect (works only if window overlaps rect border)
  //then check if window is completely in rect (if window is smaller than rect)
  let right = rect_top_left[0] + rect_dimensions[0];
  let bottom = rect_top_left[1] + rect_dimensions[1];
  return
    point_in_window(rect_top_left, top_left, dimensions) ||
    point_in_window([right, rect_top_left[1]], top_left, dimensions) ||
    point_in_window([rect_top_left[0], bottom], top_left, dimensions) ||
    point_in_window([right, bottom], top_left, dimensions) ||
    (rect_top_left[0] >= top_left[0] && rect_top_left[0] <= right && rect_top_left[1] >= top_left[1] && rect_top_left[1] <= bottom);
}
*/

lazy_static! {
  static ref WRITER: spin::Mutex<FrameBufferWriter> = spin::Mutex::new(Default::default());
  static ref WM: spin::Mutex<WindowManager> = spin::Mutex::new(Default::default());
}

pub fn init(framebuffer: FrameBuffer) {
  let framebuffer_info = framebuffer.info();
  WRITER.lock().new(framebuffer_info, framebuffer.into_buffer());
  
  //WRITER.lock().draw_rect([0, 0], [5, 5], [0, 255, 255]);

  WM.lock().init([framebuffer_info.width, framebuffer_info.height]);

  WM.lock().add_window_like(Box::new(DesktopBackground::new()), [0, 0], [framebuffer_info.width, framebuffer_info.height - TASKBAR_HEIGHT]);
  
  WM.lock().add_window_like(Box::new(Taskbar::new()), [0, framebuffer_info.height - TASKBAR_HEIGHT], [framebuffer_info.width, TASKBAR_HEIGHT]);

  without_interrupts(|| {
    WM.lock().render(None);
  });

  //
}

pub fn keyboard_emit(key_char: KeyChar) {
  unsafe { SERIAL1.lock().write_text(&format!("{:?}", &key_char)); }
  WM.lock().handle_message(WindowManagerMessage::KeyChar(key_char));
}

pub fn mouse_emit(mouse_change: MouseChange) {
  //unsafe { SERIAL1.lock().write_text(&format!("{:?}", &mouse_change)); }
  WM.lock().handle_message(WindowManagerMessage::MouseChange(mouse_change));
}

pub enum DrawInstructions {
  Rect(Point, Dimensions, RGBColor),
  Text(Point, &'static str, &'static str, RGBColor, RGBColor), //font and text
}

#[derive(PartialEq)]
pub enum WindowLikeType {
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
}

pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse;

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

impl fmt::Debug for WindowLikeInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowLikeInfo").field("id", &self.id).field("top_left", &self.top_left).field("dimensions", &self.dimensions).field("window_like", &"todo: print this out too").finish()
  }
}

#[derive(Default)]
pub struct WindowManager {
  id_count: usize,
  window_infos: Vec<WindowLikeInfo>,
  dimensions: Dimensions,
  theme: Themes,
  focused_window_id: Option<usize>,
  mouse_coords: Point,
  held_special_keys: Vec<&'static str>,
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
    let mut redraw_ids = None;
    let response: WindowMessageResponse = match message {
      WindowManagerMessage::KeyChar(key_char) => {
        //check if is special key (key releases are guaranteed to be special keys)
        //eg: ctrl, alt, command/windows, shift, or caps lock
        match key_char {
          KeyChar::Press(char) => {
            let mut press_response = WindowMessageResponse::DoNothing;
            if self.held_special_keys.contains(&"alt") {
              //keyboard shortcut
              let shortcuts = BTreeMap::from([
                ('s', ShortcutType::StartMenu),
                //
              ]);
              if let Some(shortcut) = shortcuts.get(&char) {
                if shortcut == &ShortcutType::StartMenu {
                  //send to taskbar
                  let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
                  press_response = self.window_infos[taskbar_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::StartMenu));
                } else {
                  //
                }
              }
            } else {
              //
            }
            press_response
          },
          KeyChar::SpecialPress(special_key) => {
            //add to pressed keys
            self.held_special_keys.push(special_key);
            WindowMessageResponse::DoNothing
          },
          KeyChar::SpecialRelease(special_key) => {
            //remove it from pressed keys
            let index = self.held_special_keys.iter().position(|sk| sk == &special_key).unwrap();
            self.held_special_keys.remove(index);
            WindowMessageResponse::DoNothing
          },
        }
      },
      WindowManagerMessage::MouseChange(mouse_change) => {
        if mouse_change.x_delta != 0 || mouse_change.y_delta != 0 {
          //handle mouse move
          //if the old window the mouse was on and new one differ, everything will need to be redrawn
          //no, you can't just redraw those two windows, because the mouse is not a point, it is a
          //square, so there may be infinite visible windows under the mouse, and finding and
          //redrawing those means redrawing all the windows in between those (order wise) too...
          //in other words, its easier to redraw everything
          let old_window_id_top = self.window_infos.iter().rev().find(|w| point_in_window(self.mouse_coords, w.top_left, w.dimensions)).unwrap().id;
          let old_window_id_bottom = self.window_infos.iter().rev().find(|w| point_in_window([self.mouse_coords[0], self.mouse_coords[1] + MOUSE_SIZE[1]], w.top_left, w.dimensions)).unwrap().id;
          //new mouse coords
          self.mouse_coords = [
            if -mouse_change.x_delta > self.mouse_coords[0] as i16 {
              0
            } else {
              (self.mouse_coords[0] as i16 + mouse_change.x_delta) as usize
            },
            if -mouse_change.y_delta > self.mouse_coords[1] as i16 {
              0
            } else {
              (self.mouse_coords[1] as i16 + mouse_change.y_delta) as usize
            },
          ];
          //prevent mouse going off screen
          if self.mouse_coords[0] >= (self.dimensions[0] - MOUSE_SIZE[0]) {
            self.mouse_coords[0] = self.dimensions[0] - MOUSE_SIZE[0];
          }
          if self.mouse_coords[1] >= (self.dimensions[1] - MOUSE_SIZE[1]) {
            self.mouse_coords[1] = self.dimensions[1] - MOUSE_SIZE[1];
          }
          //and the window the cursor is currently on
          let mut redraw_ids_ = Vec::new();
          let new_window_id = self.window_infos.iter().rev().find(|w| point_in_window(self.mouse_coords, w.top_left, w.dimensions)).unwrap().id;
          redraw_ids_.push(new_window_id);
          //send mouse move or mouse move outside to focused window
          let mut focus_response = WindowMessageResponse::DoNothing;
          if let Some(focused_window_id) = self.focused_window_id {
            let focused_window = &mut self.window_infos[focused_window_id];
            if point_in_window(self.mouse_coords, focused_window.top_left, focused_window.dimensions) {
              focus_response = focused_window.window_like.handle_message(WindowMessage::MouseMove(MouseMove {
                coords: [
                  self.mouse_coords[0] + focused_window.top_left[0],
                  self.mouse_coords[1] + focused_window.top_left[1],
                ],
                left: mouse_change.left,
              }));
            } else {
              focus_response = focused_window.window_like.handle_message(WindowMessage::MouseMoveOutside);
            }
            if focus_response != WindowMessageResponse::DoNothing {
              redraw_ids_.push(focused_window.id);
            }
          }
          //see way above, essentially everything will need to be redrawn if mouse just came from a different window
          if new_window_id == old_window_id_top && new_window_id == old_window_id_bottom {
            redraw_ids = Some(redraw_ids_);
          }
          //because we have to redraw the mouse, must rerender
          if focus_response == WindowMessageResponse::DoNothing {
            WindowMessageResponse::JustRerender
          } else {
            focus_response
          }
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
            let focus_response = focused_window.window_like.handle_message(WindowMessage::MouseLeftClick(MouseLeftClick {
              coords: [
                self.mouse_coords[0] - focused_window.top_left[0],
                self.mouse_coords[1] - focused_window.top_left[1],
              ],
            }));
            if changed_focus || focus_response != WindowMessageResponse::DoNothing {
              if focus_response != WindowMessageResponse::DoNothing {
                focus_response
              } else {
                WindowMessageResponse::JustRerender
              }
            } else {
              WindowMessageResponse::DoNothing
            }
          } else {
            //
            WindowMessageResponse::DoNothing
          }
        }
      },
      //
    };
    if response != WindowMessageResponse::DoNothing {
      match response {
        WindowMessageResponse::Request(request) => self.handle_request(request),
        _ => {},
      };
      self.render(redraw_ids);
    }
  }
  
  pub fn handle_request(&mut self, request: WindowManagerRequest) {
    match request {
      WindowManagerRequest::OpenWindow(w, top_left, dimensions) => {
        self.add_window_like(w, top_left, dimensions);
      },
    };
  }

  pub fn render(&mut self, maybe_redraw_ids: Option<Vec<usize>>) {
    let redraw_windows = self.window_infos.iter().filter(|w| {
      if let Some(redraw_ids) = &maybe_redraw_ids {
        redraw_ids.contains(&w.id)
      } else {
        true
      }
    });
    for window_info in redraw_windows {
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
          DrawInstructions::Text(top_left, font_name, text, color, bg_color) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            //todo: overflows and shit
            //
            WRITER.lock().draw_text(true_top_left, font_name, text, color, bg_color, 1);
          },
        }
      }
    }
    //draw mouse pointer
    //must be MOUSE_SIZE
    WRITER.lock().draw_text(self.mouse_coords, "_icons", "0", [0, 255, 0], [0, 0, 0], 0);
  }
}

