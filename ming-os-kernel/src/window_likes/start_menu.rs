use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::highlight_button::HighlightButton;


static CATEGORIES: [&'static str; 9]= ["About", "Utils", "Games", "Development", "Files", "System", "Misc", "Help", "Logout"];

#[derive(Clone)]
enum StartMenuMessage {
  CategoryClick(&'static str),
  WindowClick(&'static str),
  Back,
  ChangeAcknowledge,
}

pub struct StartMenu {
  first_draw: bool, //optimise what to redraw
  dimensions: Dimensions,
  components: Vec<Box<dyn Component<StartMenuMessage> + Send>>,
  current_focus: &'static str,
  old_focus: &'static str,
  y_each: usize,
}

impl WindowLike for StartMenu {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    self.first_draw = false;
    match message {
      WindowMessage::Init(dimensions) => {
        self.first_draw = true;
        self.dimensions = dimensions;
        self.y_each = (self.dimensions[1] - 1) / CATEGORIES.len();
        self.add_category_components();
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        //up and down
        if key_press.key == '1' || key_press.key == '2' {
          let old_focus_index = self.get_focus_index().unwrap();
          self.components[old_focus_index].handle_message(WindowMessage::Unfocus);
          let current_focus_index = if key_press.key == '2' {
              if old_focus_index + 1 == self.components.len() {
                0
              } else {
                old_focus_index + 1
              }
          } else {
            if old_focus_index == 0 {
              self.components.len() - 1
            } else {
              old_focus_index - 1
            }
          };
          self.old_focus = self.current_focus;
          self.current_focus = self.components[current_focus_index].name();
          self.components[current_focus_index].handle_message(WindowMessage::Focus);
          WindowMessageResponse::JustRerender
        } else if key_press.key == '𐘂' { //the enter key
          let focus_index = self.get_focus_index();
          if let Some(focus_index) = focus_index {
            let r = self.components[focus_index].handle_message(WindowMessage::FocusClick);
            self.handle_start_menu_message(r)
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          let current_focus_index = self.get_focus_index().unwrap();
          if let Some(n_index) = self.components[current_focus_index..].iter().position(|c| c.name().chars().next().unwrap_or('𐘂').to_lowercase().next().unwrap() == key_press.key) {
            //now old focus, not current focus
            self.components[current_focus_index].handle_message(WindowMessage::Unfocus);
            self.old_focus = self.current_focus;
            ;
            self.current_focus = self.components[current_focus_index + n_index].name();
            self.components[current_focus_index + n_index].handle_message(WindowMessage::Focus);
            WindowMessageResponse::JustRerender
          } else {
            WindowMessageResponse::DoNothing
          }
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = Vec::new();
    if self.first_draw {
      instructions = vec![
        //top thin border
        DrawInstructions::Rect([0, 0], [self.dimensions[0], 1], theme_info.border_left_top),
        //right thin border
        DrawInstructions::Rect([self.dimensions[0] - 1, 0], [1, self.dimensions[1]], theme_info.border_right_bottom),
        //background
        DrawInstructions::Rect([0, 1], [self.dimensions[0] - 1, self.dimensions[1] - 1], theme_info.background),
        //mingde logo
        DrawInstructions::Mingde([2, 2]),
        //I truly don't know why, it should be - 44 but - 30 seems to work better :shrug:
        DrawInstructions::Gradient([2, 42], [40, self.dimensions[1] - 30], [255, 201, 14], [225, 219, 77], 15),
      ];
    }
    let redraw_components = self.components.iter().filter(|c| c.name() == self.old_focus || c.name() == self.current_focus || self.first_draw);
    for component in redraw_components {
      instructions.extend(component.draw(theme_info));
    }
    instructions
  }
  
  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::StartMenu
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [175, 250]
  }
}

impl StartMenu {
  pub fn new() -> Self {
    Self {
      first_draw: true,
      dimensions: [0, 0],
      components: Vec::new(),
      current_focus: "", //placeholder, will be set in init
      old_focus: "",
      y_each: 0, //will be set in add_category_components
    }
  }

  pub fn handle_start_menu_message(&mut self, message: Option<StartMenuMessage>) -> WindowMessageResponse {
    if let Some(message) = message {
      match message {
        StartMenuMessage::CategoryClick(name) => {
          if name == "Logout" {
            WindowMessageResponse::Request(WindowManagerRequest::Lock)
          } else {
            self.first_draw = true;
            self.current_focus = "Back";
            self.components = vec![
              Box::new(HighlightButton::new(
                "Back", [42, 0], [self.dimensions[0] - 42 - 1, self.y_each + 1], "Back", StartMenuMessage::Back, StartMenuMessage::ChangeAcknowledge, true
              ))
            ];
            //add window buttons
            //
            WindowMessageResponse::JustRerender
          }
        },
        StartMenuMessage::WindowClick(name) => {
          //open the selected window
          //
          WindowMessageResponse::JustRerender
        },
        StartMenuMessage::Back => {
          self.first_draw = true;
          self.add_category_components();
          WindowMessageResponse::JustRerender
        },
        StartMenuMessage::ChangeAcknowledge => {
          //
          WindowMessageResponse::JustRerender
        },
      }
    } else {
      //maybe should be JustRerender?
      WindowMessageResponse::DoNothing
    }
  }

  pub fn add_category_components(&mut self) {
    self.current_focus = "About";
    self.components = Vec::new();
    for c in 0..CATEGORIES.len() {
      let name = CATEGORIES[c];
      self.components.push(Box::new(HighlightButton::new(
        name, [42, self.y_each * c + 1], [self.dimensions[0] - 42 - 1, self.y_each], name, StartMenuMessage::CategoryClick(name), StartMenuMessage::ChangeAcknowledge, c == 0
      )));
    }
  }

  pub fn get_focus_index(&self) -> Option<usize> {
    self.components.iter().filter(|c| c.focusable()).position(|c| c.name() == self.current_focus)
  }
}

