use std::env;
use std::fs::{ read_dir, write };
use std::path::Path;

use bmp_rust::bmp::BMP;

fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("bmp.rs");
  let mut times_new_roman: Vec<(char, Vec<Vec<u8>>, u8)> = Vec::new();
  for entry in read_dir("./bmps/times-new-roman").unwrap() {
    let path = entry.unwrap().path();
    let mut ch: Vec<Vec<u8>> = Vec::new();
    if !path.is_dir() {
      let b = BMP::new_from_file(&path.clone().into_os_string().into_string().unwrap());
      let dib_header = b.get_dib_header().unwrap();
      let width = dib_header.width as usize;
      let height = dib_header.height as usize;
      for y in 0..height {
        let mut row = Vec::new();
        for x in 0..width {
          let pixel_color = b.get_color_of_px(x, y).unwrap();
          println!("{:?}", pixel_color);
          //if black, true
          row.push(pixel_color[3]); //push alpha channel
        }
        ch.push(row);
      }
      let path_chars: Vec<char> = path.file_name().unwrap().to_str().unwrap().to_string().chars().collect();
      times_new_roman.push((path_chars[0], ch, path_chars[1].to_digit(10).unwrap() as u8));
    }
  }
  let fonts = vec![
    ("times-new-roman", times_new_roman),
  ];
  let type_annotation = "Vec<(&'static str, Vec<(char, Vec<Vec<u8>>, u8)>)>"; //manually changed every time
  write(
    &dest_path,
    format!("use alloc::vec;\npub fn get_fonts() -> {} {{\n  {}\n}}\n", type_annotation, format!("{:?}", fonts).replace("[", "vec!["))
  ).unwrap();
}

