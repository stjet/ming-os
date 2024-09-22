use alloc::vec::Vec;

use bootloader_api::info::{ FrameBufferInfo, PixelFormat };
use lazy_static::lazy_static;

include!(concat!(env!("OUT_DIR"), "/bmp.rs"));

pub type Point = [usize; 2]; //x, y
pub type Dimensions = [usize; 2]; //width, height
pub type RGBColor = [u8; 3]; //rgb

type Font = (&'static str, Vec<(char, Vec<Vec<u8>>, u8)>, u8);
type Fonts = Vec<Font>; //...yeah

lazy_static! {
  pub static ref FONTS: Fonts = get_fonts();
}

pub fn get_font_max_height(font_name: &'static str) -> Option<u8> {
  for font in &*FONTS {
    if font.0 == font_name {
      return Some(font.2);
    }
  }
  return None;
}

fn color_with_alpha(color: RGBColor, bg_color: RGBColor, alpha: u8) -> RGBColor {
  /*let factor: f32 = alpha as f32 / 255.0;
  [
    (bg_color[0] as f32 * (1.0 - factor)) as u8 + (color[0] as f32 * factor) as u8,
    (bg_color[1] as f32 * (1.0 - factor)) as u8 + (color[1] as f32 * factor) as u8,
    (bg_color[2] as f32 * (1.0 - factor)) as u8 + (color[2] as f32 * factor) as u8,
  ]*/
  //255 * 255 < max(u16)
  let alpha = alpha as u16;
  [
    (bg_color[0] as u16 * (255 - alpha) / 255) as u8 + (color[0] as u16 * alpha / 255) as u8,
    (bg_color[1] as u16 * (255 - alpha) / 255) as u8 + (color[1] as u16 * alpha / 255) as u8,
    (bg_color[2] as u16 * (255 - alpha) / 255) as u8 + (color[2] as u16 * alpha / 255) as u8,
  ]
}

//currently doesn't check if writing onto next line accidentally

pub struct FrameBufferWriter {
  info: FrameBufferInfo,
  raw_buffer: Option<&'static mut [u8]>,
}

impl Default for FrameBufferWriter {
  fn default() -> Self {
    Self {
      raw_buffer: None,
      info: FrameBufferInfo {
        byte_len: 0,
        width: 0,
        height: 0,
        pixel_format: PixelFormat::Rgb,
        bytes_per_pixel: 0,
        stride: 0,
      }
    }
  }
}

impl FrameBufferWriter {
  pub fn new(&mut self, info: FrameBufferInfo, raw_buffer: &'static mut [u8]) {
    self.info = info;
    self.raw_buffer = Some(raw_buffer);
  }

  fn _draw_pixel(&mut self, start_pos: usize, color: RGBColor) {
    let color = match self.info.pixel_format {
      PixelFormat::Rgb => color,
      PixelFormat::Bgr => [color[2], color[1], color[0]],
      _ => panic!("Not rgb or bgr"),
    };
    self.raw_buffer.as_mut().unwrap()[start_pos..(start_pos + 3)].copy_from_slice(&color[..]);
  }

  fn _draw_line(&mut self, start_pos: usize, bytes: &[u8]) {
    self.raw_buffer.as_mut().unwrap()[start_pos..(start_pos + bytes.len())]
      .copy_from_slice(bytes);
  }

  pub fn _draw_char(&mut self, top_left: Point, font: &Font, c: char, color: RGBColor, bg_color: RGBColor) -> Option<usize> {
    for tri in &font.1 {
      if tri.0 == c {
        let mut start_pos;
        for row in 0..tri.1.len() {
          //tri.2 is vertical offset
          start_pos = ((top_left[1] + row + tri.2 as usize) * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
          for col in &tri.1[row] {
            if col > &0 {
              self._draw_pixel(start_pos, color_with_alpha(color, bg_color, *col));
            }
            start_pos += self.info.bytes_per_pixel;
          }
        }
        //returns char width
        return Some(tri.1[0].len());
      }
    }
    return None;
  }

  //dots

  pub fn draw_pixel(&mut self, point: Point, color: RGBColor) {
    let start_pos = (point[1] * self.info.stride + point[0]) * self.info.bytes_per_pixel;
    self._draw_pixel(start_pos, color);
  }
  
  //(lines are rectangles of height 1)
  pub fn draw_line(&mut self, left: Point, width: usize, color: RGBColor) {
    self.draw_rect(left, [width, 1], color);
  }

  //shapes

  pub fn draw_rect(&mut self, top_left: Point, dimensions: Dimensions, color: RGBColor) {
    let color = match self.info.pixel_format {
      PixelFormat::Rgb => color,
      PixelFormat::Bgr => [color[2], color[1], color[0]],
      _ => panic!("Not rgb or bgr"),
    };
    let line_bytes = if self.info.bytes_per_pixel > 3 {
      [color[0], color[1], color[2], 255].repeat(dimensions[0])
    } else {
      color.repeat(dimensions[0])
    };
    let mut start_pos = (top_left[1] * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
    for _row in 0..dimensions[1] {
      /*
       * for _col in 0..dimensions[0] {
        self._draw_pixel(start_pos, color);
        start_pos += self.info.bytes_per_pixel;
      }
      //assumes stride is same as bytes_per_pixel * width
      //start_pos = start_pos + top_left[0] * self.info.bytes_per_pixel;
      */
      //use _draw_line instead for MUCH more efficiency
      self._draw_line(start_pos, &line_bytes[..]);
      start_pos += self.info.stride * self.info.bytes_per_pixel;
    }
  }

  //text

  pub fn draw_text(&mut self, top_left: Point, font_name: &str, text: &str, color: RGBColor, bg_color: RGBColor, horiz_spacing: usize) {
    let mut top_left = top_left;
    for font in &*FONTS {
      if font.0 == font_name {
        for c in text.chars() {
          let char_width = self._draw_char(top_left, &font, c, color, bg_color).unwrap();
          top_left[0] = top_left[0] + char_width + horiz_spacing;
        }
      }
    }
  }
}

