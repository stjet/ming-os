use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::fmt;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::format;

use spin;
use bootloader_api::info::FrameBuffer;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;

use crate::framebuffer::{ FrameBufferWriter, Point, Dimensions, RGBColor };
use crate::window_likes::desktop_background::DesktopBackground;
use crate::window_likes::taskbar::Taskbar;
use crate::window_likes::lock_screen::LockScreen;
use crate::window_likes::workspace_indicator::WorkspaceIndicator;
use crate::themes::{ ThemeInfo, Themes, get_theme_info };
use crate::keyboard::{ KeyChar, uppercase_or_special };
use crate::SERIAL1;
use crate::messages::*;

pub const TASKBAR_HEIGHT: usize = 38;
pub const INDICATOR_HEIGHT: usize = 20;
pub const WINDOW_TOP_HEIGHT: usize = 26;
static mut DEBUG_COUNTER: usize = 0;

lazy_static! {
  static ref WRITER: spin::Mutex<FrameBufferWriter<'static>> = spin::Mutex::new(Default::default());
  static ref WM: spin::Mutex<WindowManager> = spin::Mutex::new(Default::default());
}

pub fn init(framebuffer: FrameBuffer) {
  let framebuffer_info = framebuffer.info();
  WRITER.lock().new(framebuffer_info, framebuffer.into_buffer());
  
  WM.lock().init([framebuffer_info.width, framebuffer_info.height]);

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
  //unsafe { SERIAL1.lock().write_text(&format!("{:?}", &kc)); }
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

#[derive(Debug)]
pub enum DrawInstructions {
  Rect(Point, Dimensions, RGBColor),
  Text(Point, &'static str, String, RGBColor, RGBColor), //font and text
  Gradient(Point, Dimensions, RGBColor, RGBColor, usize),
  Mingde(Point),
}

#[derive(Debug, PartialEq)]
pub enum WindowLikeType {
  LockScreen,
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
  WorkspaceIndicator,
}

pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse;

  //properties
  fn title(&self) -> &'static str {
    ""
  }
  fn subtype(&self) -> WindowLikeType;
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions; //needs &self or its not object safe or some bullcrap
}

pub enum Workspace {
  All,
  Workspace(u8), //goes from 0-8
}

pub struct WindowLikeInfo {
  id: usize,
  window_like: Box<dyn WindowLike + Send>,
  top_left: Point,
  dimensions: Dimensions,
  workspace: Workspace,
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
  held_special_keys: Vec<&'static str>,
  locked: bool,
  current_workspace: u8,
}

impl WindowManager {
  pub fn init(&mut self, dimensions: Dimensions) {
    self.dimensions = dimensions;
    self.lock();
  }

  pub fn add_window_like(&mut self, mut window_like: Box<dyn WindowLike + Send>, top_left: Point, dimensions: Option<Dimensions>) {
    let subtype = window_like.subtype();
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
      workspace: if subtype == WindowLikeType::Window {
        Workspace::Workspace(self.current_workspace)
      } else {
        Workspace::All
      },
    });
  }

  fn get_focused_index(&self) -> Option<usize> {
    self.window_infos.iter().position(|w| w.id == self.focused_id)
  }

  //should return an iterator but fuck it!
  fn get_windows_in_workspace(&self) -> Vec<&WindowLikeInfo> {
    self.window_infos.iter().filter(|w| {
      match w.workspace {
        Workspace::Workspace(workspace) => workspace == self.current_workspace,
        _ => false, //filter out taskbar, indicator, background, start menu, etc
      }
    }).collect()
  }

  fn lock(&mut self) {
    self.locked = true;
    self.window_infos = Vec::new();
    self.add_window_like(Box::new(LockScreen::new()), [0, 0], None);
  }

  fn unlock(&mut self) {
    self.locked = false;
    self.window_infos = Vec::new();
    self.add_window_like(Box::new(DesktopBackground::new()), [0, INDICATOR_HEIGHT], None);
    self.add_window_like(Box::new(Taskbar::new()), [0, self.dimensions[1] - TASKBAR_HEIGHT], None);
    self.add_window_like(Box::new(WorkspaceIndicator::new()), [0, 0], None);
  }

  //if off_only is true, also handle request
  fn toggle_start_menu(&mut self, off_only: bool) -> WindowMessageResponse {
    let start_menu_exists = self.window_infos.iter().find(|w| w.window_like.subtype() == WindowLikeType::StartMenu).is_some();
    if (start_menu_exists && off_only) || !off_only {
      let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
      self.focused_id = self.window_infos[taskbar_index].id;
      if off_only {
        self.handle_request(WindowManagerRequest::CloseStartMenu);
      }
      self.window_infos[taskbar_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::StartMenu))
    } else {
      WindowMessageResponse::DoNothing
    }
  }

  fn taskbar_update_windows(&mut self) {
    let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
    let message = WindowMessage::Info(InfoType::WindowsInWorkspace(
      self.get_windows_in_workspace().iter().map(|w| (w.id, w.window_like.title())).collect(),
      self.focused_id
    ));
    self.window_infos[taskbar_index].window_like.handle_message(message);
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
            if self.held_special_keys.contains(&"alt") && !self.locked {
              //keyboard shortcut
              let shortcuts = BTreeMap::from([
                ('s', ShortcutType::StartMenu),
                ('1', ShortcutType::SwitchWorkspace(0)),
                ('2', ShortcutType::SwitchWorkspace(1)),
                ('3', ShortcutType::SwitchWorkspace(2)),
                ('4', ShortcutType::SwitchWorkspace(3)),
                ('5', ShortcutType::SwitchWorkspace(4)),
                ('6', ShortcutType::SwitchWorkspace(5)),
                ('7', ShortcutType::SwitchWorkspace(6)),
                ('8', ShortcutType::SwitchWorkspace(7)),
                ('9', ShortcutType::SwitchWorkspace(8)),
                (']', ShortcutType::FocusNextWindow),
                ('q', ShortcutType::QuitWindow),
                //
              ]);
              if let Some(shortcut) = shortcuts.get(&c) {
                match shortcut {
                  &ShortcutType::StartMenu => {
                    //send to taskbar
                    press_response = self.toggle_start_menu(false);
                  },
                  &ShortcutType::SwitchWorkspace(workspace) => {
                    if self.current_workspace != workspace {
                      //close start menu if open
                      self.toggle_start_menu(true);
                      self.current_workspace = workspace;
                      //send to workspace indicator
                      let indicator_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::WorkspaceIndicator).unwrap();
                      self.focused_id = self.window_infos[indicator_index].id;
                      self.window_infos[indicator_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::SwitchWorkspace(self.current_workspace)));
                      self.taskbar_update_windows();
                      press_response = WindowMessageResponse::JustRerender;
                    }
                  },
                  &ShortcutType::FocusNextWindow => {
                    let current_index = self.get_focused_index().unwrap_or(0);
                    let mut new_focus_index = current_index;
                    loop {
                      new_focus_index += 1;
                      if new_focus_index == self.window_infos.len() {
                        new_focus_index = 0;
                      }
                      if self.window_infos[new_focus_index].window_like.subtype() == WindowLikeType::Window {
                        //switch focus to this
                        self.focused_id = self.window_infos[new_focus_index].id;
                        self.taskbar_update_windows();
                        //todo: elevate it to the top
                        //
                        press_response = WindowMessageResponse::JustRerender;
                        break;
                      } else if new_focus_index == current_index {
                        break; //did a full loop, found no windows
                      }
                    }
                  },
                  &ShortcutType::QuitWindow => {
                    if let Some(focused_index) = self.get_focused_index() {
                      if self.window_infos[focused_index].window_like.subtype() == WindowLikeType::Window {
                        self.window_infos.remove(focused_index);
                        self.taskbar_update_windows();
                        press_response = WindowMessageResponse::JustRerender;
                      }
                    }
                  },
                };
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
    let focused_index = self.get_focused_index().unwrap();
    let subtype = self.window_infos[focused_index].window_like.subtype();
    match request {
      WindowManagerRequest::OpenWindow(w) => {
        if subtype != WindowLikeType::Taskbar && subtype != WindowLikeType::StartMenu {
          return;
        }
        //close start menu if open
        self.toggle_start_menu(true);
        let ideal_dimensions = w.ideal_dimensions(self.dimensions);
        let top_left = match w.subtype() {
          WindowLikeType::StartMenu => [0, self.dimensions[1] - TASKBAR_HEIGHT - ideal_dimensions[1]],
          WindowLikeType::Window => [42, 42],
          _ => [0, 0],
        };
        self.add_window_like(w, top_left, Some(ideal_dimensions));
        self.taskbar_update_windows();
      },
      WindowManagerRequest::CloseStartMenu => {
        if subtype != WindowLikeType::Taskbar && subtype != WindowLikeType::StartMenu {
          return;
        }
        let start_menu_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::StartMenu);
        if let Some(start_menu_index) = start_menu_index {
          self.window_infos.remove(start_menu_index);
        }
      },
      WindowManagerRequest::Unlock => {
        if subtype != WindowLikeType::LockScreen {
          return;
        }
        self.unlock();
      },
      WindowManagerRequest::Lock => {
        if subtype != WindowLikeType::StartMenu {
          return;
        }
        self.lock();
      },
    };
  }

  //another issue with a huge vector of draw instructions; it takes up heap memory
  pub fn render(&mut self, maybe_redraw_ids: Option<Vec<usize>>) {
    let redraw_windows = self.window_infos.iter().filter(|w| {
      if let Some(redraw_ids) = &maybe_redraw_ids {
        redraw_ids.contains(&w.id)
      } else {
        true
      }
    }).filter(|w| {
      match w.workspace {
        Workspace::Workspace(workspace) => workspace == self.current_workspace,
        _ => true,
      }
    });
    let theme_info = get_theme_info(&self.theme).unwrap();
    for window_info in redraw_windows {
      //unsafe { SERIAL1.lock().write_text(&format!("{:?}\n", &window_info.window_like.subtype())); }
      let mut instructions = Vec::new();
      if window_info.window_like.subtype() == WindowLikeType::Window {
        //draw window background
        instructions.push(DrawInstructions::Rect([0, 0], window_info.dimensions, theme_info.background));
      }
      instructions.extend(window_info.window_like.draw(&theme_info));
      if window_info.window_like.subtype() == WindowLikeType::Window {
        //draw window top decorations and what not
        instructions.extend(vec![
          //left top border
          DrawInstructions::Rect([0, 0], [window_info.dimensions[0], 1], theme_info.border_left_top),
          DrawInstructions::Rect([0, 0], [1, window_info.dimensions[1]], theme_info.border_left_top),
          //top
          DrawInstructions::Rect([1, 1], [window_info.dimensions[0] - 2, WINDOW_TOP_HEIGHT - 3], theme_info.top),
          DrawInstructions::Text([4, 4], "times-new-roman", window_info.window_like.title().to_string(), theme_info.text_top, theme_info.top),
          //top bottom border
          DrawInstructions::Rect([1, WINDOW_TOP_HEIGHT - 2], [window_info.dimensions[0] - 2, 2], theme_info.border_left_top),
          //right bottom border
          DrawInstructions::Rect([window_info.dimensions[0] - 1, 1], [1, window_info.dimensions[1] - 1], theme_info.border_right_bottom),
          DrawInstructions::Rect([1, window_info.dimensions[1] - 1], [window_info.dimensions[0] - 1, 1], theme_info.border_right_bottom),
        ]);
      }
      let mut window_writer: FrameBufferWriter = Default::default();
      let mut framebuffer_info = WRITER.lock().info;
      let window_width = window_info.dimensions[0];
      let window_height = window_info.dimensions[1];
      framebuffer_info.width = window_width;
      framebuffer_info.height = window_height;
      framebuffer_info.stride = window_width;
      let mut temp_vec = vec![0 as u8; window_width * window_height * framebuffer_info.bytes_per_pixel];
      //make a writer just for the window
      window_writer.new(framebuffer_info, &mut temp_vec[..]);
      for instruction in instructions {
        //unsafe { SERIAL1.lock().write_text(&format!("{:?}\n", instruction)); }
        match instruction {
          DrawInstructions::Rect(top_left, dimensions, color) => {
            //try and prevent overflows out of the window
            let true_dimensions = [
              min(dimensions[0], window_info.dimensions[0] - top_left[0]),
              min(dimensions[1], window_info.dimensions[1] - top_left[1]),
            ];
            window_writer.draw_rect(top_left, true_dimensions, color);
          },
          DrawInstructions::Text(top_left, font_name, text, color, bg_color) => {
            //todo: overflows and shit
            //
            window_writer.draw_text(top_left, font_name, &text, color, bg_color, 1);
          },
          DrawInstructions::Mingde(top_left) => {
            //todo: overflows and shit
            //
            window_writer._draw_mingde(top_left);
          },
          DrawInstructions::Gradient(top_left, dimensions, start_color, end_color, steps) => {
            //todo: overflows and shit
            //
            window_writer.draw_gradient(top_left, dimensions, start_color, end_color, steps);
          },
        }
      }
      WRITER.lock().draw_buffer(window_info.top_left, window_info.dimensions[1], window_info.dimensions[0] * framebuffer_info.bytes_per_pixel, &window_writer.get_buffer());
      //core::mem::drop(window_writer);
      //core::mem::drop(temp_vec);
    }
  }
}

