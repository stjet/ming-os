use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, TASKBAR_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest, ShortcutType };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::toggle_button::{ ToggleButton, ToggleButtonAlignment };
use crate::window_likes::start_menu::StartMenu;

const PADDING: usize = 4;

#[derive(Clone)]
enum TaskbarMessage {
  ShowStartMenu,
  HideStartMenu,
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
          Box::new(ToggleButton::new("start-button", [PADDING, PADDING], [44, self.dimensions[1] - (PADDING * 2)], "Start", TaskbarMessage::ShowStartMenu, TaskbarMessage::HideStartMenu, false, Some(ToggleButtonAlignment::Left))),
        ];
        WindowMessageResponse::JustRerender
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::StartMenu => {
            let start_index = self.components.iter().position(|c| c.name() == "start-button").unwrap();
            let start_response = self.components[start_index].handle_message(WindowMessage::FocusClick);
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
      DrawInstructions::Rect([0, 0], [self.dimensions[0], 1], theme_info.border_left_top),
      //the actual taskbar background
      DrawInstructions::Rect([0, 1], [self.dimensions[0], self.dimensions[1] - 1], theme_info.background),
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

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0], TASKBAR_HEIGHT]
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
        TaskbarMessage::ShowStartMenu => {
          //todo: fix
          WindowMessageResponse::Request(WindowManagerRequest::OpenWindow(Box::new(StartMenu::new())))
        },
        TaskbarMessage::HideStartMenu => {
          WindowMessageResponse::Request(WindowManagerRequest::CloseStartMenu)
        },
      }
    } else {
      //maybe should be JustRerender?
      WindowMessageResponse::DoNothing
    }
  }
}


