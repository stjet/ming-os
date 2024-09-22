use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest, ShortcutType, MouseLeftClick };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::button::{ Button, ButtonAlignment };
use crate::SERIAL1;
use crate::window_likes::start_menu::StartMenu;

const PADDING: usize = 4;

#[derive(Clone)]
enum TaskbarMessage {
  ToggleStartMenu,
  //
}

pub struct Taskbar {
  dimensions: Dimensions,
  components: Vec<Box<dyn Component<TaskbarMessage> + Send>>,
}

impl WindowLike for Taskbar {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.components = vec![
          Box::new(Button::new("start-button", [PADDING, PADDING], [44, self.dimensions[1] - (PADDING * 2)], "Start", TaskbarMessage::ToggleStartMenu, false, Some(ButtonAlignment::Left))),
        ];
        WindowMessageResponse::JustRerender
      },
      WindowMessage::MouseLeftClick(left_click) => {
        let clicked_index = self.components.iter().rposition(|c| c.clickable() && c.point_inside(left_click.coords));
        if let Some(clicked_index) = clicked_index {
          if let Some(taskbar_message) = self.components[clicked_index].handle_message(WindowMessage::MouseLeftClick(left_click)) {
            return self.handle_taskbar_message(Some(taskbar_message));
          }
        }
        WindowMessageResponse::DoNothing
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::StartMenu => {
            unsafe { SERIAL1.lock().write_text("aaa"); }
            let start_index = self.components.iter().position(|c| c.name() == "start-button").unwrap();
            let start_response = self.components[start_index].handle_message(WindowMessage::MouseLeftClick(MouseLeftClick { coords: [0, 0] }));
            self.handle_taskbar_message(start_response) //dummy left click event
          }
          //
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  //simple
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      //top thin white border
      DrawInstructions::Rect([0, 0], [self.dimensions[0], 2], theme_info.border_left_top),
      //the actual taskbar background
      DrawInstructions::Rect([0, 2], [self.dimensions[0], self.dimensions[1] - 2], theme_info.background),
    ];
    for component in &self.components {
      instructions.extend(component.draw(theme_info));
    }
    instructions
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Taskbar
  }
}

impl Taskbar {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
      components: Vec::new(),
    }
  }

  pub fn handle_taskbar_message(&mut self, message: Option<TaskbarMessage>) -> WindowMessageResponse {
    if let Some(message) = message {
      match message {
        TaskbarMessage::ToggleStartMenu => {
          //todo: fix
          WindowMessageResponse::Request(WindowManagerRequest::OpenWindow(Box::new(StartMenu::new()), [10, 10], [100, 100]))
        },
        //
      }
    } else {
      //maybe should be JustRerender?
      WindowMessageResponse::DoNothing
    }
  }
}


