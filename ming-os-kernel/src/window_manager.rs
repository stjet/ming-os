use core::panic::PanicInfo;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;

use spin;
use bootloader_api::info::{ FrameBuffer, FrameBufferInfo };
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;

use crate::framebuffer:: { FrameBufferWriter, Point, Dimensions, RGBColor };
use crate::window_likes::desktop_background::DesktopBackground;
use crate::hlt_loop;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  WRITER.lock().draw_rect([0, 0], [200, 25], [0, 255, 255]);
  WRITER.lock().draw_text([0, 0], "times-new-roman", &format!("{}", info), [0, 0, 0], [0, 255, 255], 0);
  hlt_loop();
}

lazy_static! {
  static ref WRITER: spin::Mutex<FrameBufferWriter> = spin::Mutex::new(Default::default());
}

pub fn init(framebuffer: FrameBuffer) {
  let framebuffer_info = framebuffer.info();
  WRITER.lock().new(framebuffer_info, framebuffer.into_buffer());
  
  //WRITER.lock().draw_rect([0, 0], [5, 5], [0, 255, 255]);

  let mut wm = WindowManager::new(framebuffer_info);

  wm.add_window_like(Box::new(DesktopBackground::new()), [0, 0], [framebuffer_info.width, framebuffer_info.height]);

  without_interrupts(|| {
    wm.render();
  });
  //
}

pub enum DrawInstructions {
  Rect(Point, Dimensions, RGBColor),
}

pub enum WindowLikeType {
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
}

pub enum WindowMessage {
  Init(Dimensions),
  //
}

pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> bool;

  //properties
  fn subtype(&self) -> WindowLikeType;
  fn draw(&self) -> Vec<DrawInstructions>;
}

pub struct WindowLikeInfo {
  id: usize,
  window_like: Box<dyn WindowLike>,
  top_left: Point,
  dimensions: Dimensions,
}

pub struct WindowManager {
  id_count: usize,
  window_infos: Vec<WindowLikeInfo>,
  info: FrameBufferInfo,
}

impl WindowManager {
  pub fn new(info: FrameBufferInfo) -> Self {
    Self {
      id_count: 0,
      window_infos: Vec::new(),
      info, 
    }
  }

  pub fn add_window_like(&mut self, mut window_like: Box<dyn WindowLike>, top_left: Point, dimensions: Dimensions) {
    self.id_count = self.id_count + 1;
    let id = self.id_count;
    window_like.handle_message(WindowMessage::Init(dimensions));
    self.window_infos.push(WindowLikeInfo {
      id,
      window_like,
      top_left,
      dimensions,
    });
    //
  }

  //

  pub fn handle_message(&self, message: WindowMessage) {
    match message {
      //
      _ => {},
    }
  }

  pub fn render(&mut self) {
    for window_info in &self.window_infos {
      //draw window decorations and what not
      //
      //
      for instruction in window_info.window_like.draw() {
        match instruction {
          DrawInstructions::Rect(top_left, dimensions, color) => {
            let true_top_left = [top_left[0] + window_info.top_left[0], top_left[1] + window_info.top_left[1]];
            let true_dimensions = [
              if dimensions[0] > window_info.dimensions[0] { window_info.dimensions[0] } else { dimensions[0] },
              if dimensions[1] > window_info.dimensions[1] { window_info.dimensions[1] } else { dimensions[1] },
            ];
            WRITER.lock().draw_rect(true_top_left, true_dimensions, color);
          },

          //_ => {},
        }
      }
    }
  }
}

