use alloc::vec::Vec;
use alloc::vec;
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
use crate::keyboard::{ KeyChar, uppercase_or_special };
use crate::SERIAL1;
use crate::messages::*;

pub const TASKBAR_HEIGHT: usize = 38;
static mut DEBUG_COUNTER: usize = 0;

lazy_static! {
  static ref WRITER: spin::Mutex<FrameBufferWriter> = spin::Mutex::new(Default::default());
  static ref WM: spin::Mutex<WindowManager> = spin::Mutex::new(Default::default());
}

pub fn init(framebuffer: FrameBuffer) {
  let framebuffer_info = framebuffer.info();
  WRITER.lock().new(framebuffer_info, framebuffer.into_buffer());
  
  //WRITER.lock().draw_rect([0, 0], [5, 5], [0, 255, 255]);

  WM.lock().init([framebuffer_info.width, framebuffer_info.height]);

  WM.lock().add_window_like(Box::new(DesktopBackground::new()), [0, 0], None);
  
  WM.lock().add_window_like(Box::new(Taskbar::new()), [0, framebuffer_info.height - TASKBAR_HEIGHT], None);

  without_interrupts(|| {
    WM.lock().render(None);
  });

  //
}

pub fn min(one: usize, two: usize) -> usize {
  if one > two { two } else { one } 
}

pub fn keyboard_emit(key_char: KeyChar) {
  let mut kc = key_char;
  if let KeyChar::Press(c) = kc {
    if WM.lock().held_special_keys.contains(&"shift") {
      kc = KeyChar::Press(uppercase_or_special(c));
    }
  }
  unsafe { SERIAL1.lock().write_text(&format!("{:?}", &kc)); }
  WM.lock().handle_message(WindowManagerMessage::KeyChar(kc));
}

pub fn draw_panic(p: &str) {
  WRITER.lock().draw_rect([0, 0], [200, 10], [0, 255, 0]);
  WRITER.lock().draw_text([0, 0], "times-new-roman", p, [0, 0, 0], [0, 255, 0], 1);
}

pub fn debug_write() {
  let color = match unsafe { DEBUG_COUNTER } % 3 {
    0 => [255, 0, 0],
    1 => [0, 255, 0],
    _ => [0, 0, 255], //2
  };
  WRITER.lock().draw_rect([150 + unsafe { DEBUG_COUNTER } * 4, 150], [4, 4], color);
  unsafe {
    DEBUG_COUNTER += 1;
  }
}

pub enum DrawInstructions {
  Rect(Point, Dimensions, RGBColor),
  Text(Point, &'static str, &'static str, RGBColor, RGBColor), //font and text
  Gradient(Point, Dimensions, RGBColor, RGBColor, usize),
  Mingde(Point),
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

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions; //needs &self or its not object safe or some bullcrap
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
  focused_id: usize,
  //mouse_coords: Point,
  held_special_keys: Vec<&'static str>,
}

impl WindowManager {
  pub fn init(&mut self, dimensions: Dimensions) {
    self.dimensions = dimensions;
  }

  pub fn add_window_like(&mut self, mut window_like: Box<dyn WindowLike + Send>, top_left: Point, dimensions: Option<Dimensions>) {
    let dimensions = dimensions.unwrap_or(window_like.ideal_dimensions(self.dimensions));
    self.id_count = self.id_count + 1;
    let id = self.id_count;
    self.focused_id = id;
    window_like.handle_message(WindowMessage::Init(dimensions));
    self.window_infos.push(WindowLikeInfo {
      id,
      window_like,
      top_left,
      dimensions,
    });
  }

  fn get_focused_index(&self) -> Option<usize> {
    self.window_infos.iter().position(|w| w.id == self.focused_id)
  }

  pub fn handle_message(&mut self, message: WindowManagerMessage) {
    let mut redraw_ids = None;
    let response: WindowMessageResponse = match message {
      WindowManagerMessage::KeyChar(key_char) => {
        //check if is special key (key releases are guaranteed to be special keys)
        //eg: ctrl, alt, command/windows, shift, or caps lock
        match key_char {
          KeyChar::Press(c) => {
            let mut press_response = WindowMessageResponse::DoNothing;
            if self.held_special_keys.contains(&"alt") {
              //keyboard shortcut
              let shortcuts = BTreeMap::from([
                ('s', ShortcutType::StartMenu),
                //
              ]);
              if let Some(shortcut) = shortcuts.get(&c) {
                if shortcut == &ShortcutType::StartMenu {
                  //send to taskbar
                  let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
                  press_response = self.window_infos[taskbar_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::StartMenu))
                } else {
                  //
                }
              }
            }
            //not a shortcut, basically. a regular key press
            if press_response == WindowMessageResponse::DoNothing {
              //send to focused window
              if let Some(focused_index) = self.get_focused_index() {
                press_response = self.window_infos[focused_index].window_like.handle_message(WindowMessage::KeyPress(KeyPress {
                  key: c,
                  held_special_keys: self.held_special_keys.clone(),
                }));
                //at most, only the focused window needs to be rerendered
                redraw_ids = Some(vec![self.window_infos[focused_index].id]);
              }
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
      //
    };
    //requests can result in window openings and closings, etc
    if response != WindowMessageResponse::JustRerender {
      redraw_ids = None;
    }
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
      WindowManagerRequest::OpenWindow(w) => {
        let ideal_dimensions = w.ideal_dimensions(self.dimensions);
        let top_left = match w.subtype() {
          WindowLikeType::StartMenu => [0, self.dimensions[1] - TASKBAR_HEIGHT - ideal_dimensions[1]],
          _ => [0, 0],
        };
        self.add_window_like(w, top_left, Some(ideal_dimensions));
      },
      WindowManagerRequest::CloseStartMenu => {
        let start_menu_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::StartMenu);
        if let Some(start_menu_index) = start_menu_index {
          self.window_infos.remove(start_menu_index);
        }
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
            //try and prevent overflows out of the window
            let true_dimensions = [
              min(dimensions[0], window_info.dimensions[0] - top_left[0]),
              min(dimensions[1], window_info.dimensions[1] - top_left[1]),
            ];
            WRITER.lock().draw_rect(true_top_left, true_dimensions, color);
          },
          DrawInstructions::Text(top_left, font_name, text, color, bg_color) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            //todo: overflows and shit
            //
            WRITER.lock().draw_text(true_top_left, font_name, text, color, bg_color, 1);
          },
          DrawInstructions::Mingde(top_left) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            //todo: overflows and shit
            //
            WRITER.lock()._draw_mingde(true_top_left);
          },
          DrawInstructions::Gradient(top_left, dimensions, start_color, end_color, steps) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            //todo: overflows and shit
            //
            WRITER.lock().draw_gradient(true_top_left, dimensions, start_color, end_color, steps);
          },
        }
      }
    }
  }
}

