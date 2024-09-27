use std::env;
use std::fs::{ read_dir, write };
use std::path::Path;

use bmp_rust::bmp::BMP;

fn get_font(dir: &str) -> Vec<(char, Vec<Vec<u8>>, u8)> {
  let mut font: Vec<(char, Vec<Vec<u8>>, u8)> = Vec::new();
  for entry in read_dir(dir).unwrap() {
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
      font.push((path_chars[0], ch, path_chars[1].to_digit(10).unwrap() as u8));
    }
  }
  font
}

//the Vec<u8> should be [u8; 3] but thats a job for another day
fn get_bmp(path: &str) -> Vec<Vec<Vec<u8>>> {
  let mut bmp: Vec<Vec<Vec<u8>>> = Vec::new();
  let b = BMP::new_from_file(path);
  let dib_header = b.get_dib_header().unwrap();
  let width = dib_header.width as usize;
  let height = dib_header.height as usize;
  for y in 0..height {
    let mut row = Vec::new();
    for x in 0..width {
      let pixel_color = b.get_color_of_px(x, y).unwrap();
      //if black, true
      row.push(vec![pixel_color[0], pixel_color[1], pixel_color[2]]); //push alpha channel
    }
    bmp.push(row);
  }
  bmp
}

fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("bmp.rs");
  //last member of tuple is max height
  //todo: get font max height programatically, should be pretty easy
  let fonts = vec![
    ("times-new-roman", get_font("./bmps/times-new-roman"), 12),
  ];
  let type_annotation1 = "Vec<(&'static str, Vec<(char, Vec<Vec<u8>>, u8)>, u8)>";
  let bmps = get_bmp("./bmps/mingde.bmp");
  let type_annotation2 = "[[[u8; 3]; 40]; 40]";
  write(
    &dest_path,
    format!("use alloc::vec;\npub fn get_fonts() -> {} {{\n  {}\n}}\npub fn get_mingde() -> {} {{\n  {}\n}}", type_annotation1, format!("{:?}", fonts).replace("[", "vec!["), type_annotation2, format!("{:?}", bmps))
  ).unwrap();
}

