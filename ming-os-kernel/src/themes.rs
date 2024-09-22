use crate::framebuffer::RGBColor;

#[derive(PartialEq, Default)]
pub enum Themes {
  #[default]
  Standard,
  //
}

pub struct ThemeInfo {
  pub background: RGBColor,
  pub border_left_top: RGBColor,
  pub border_right_bottom: RGBColor,
  pub text: RGBColor,
  //
}

const THEME_INFOS: [(Themes, ThemeInfo); 1] = [
  (Themes::Standard, ThemeInfo {
    background: [192, 192, 192],
    border_left_top: [255, 255, 255],
    border_right_bottom: [0, 0, 0],
    text: [0, 0, 0],
    //
  }),
];

pub fn get_theme_info(theme: &Themes) -> Option<ThemeInfo> {
  for pair in THEME_INFOS {
    if &pair.0 == theme {
      return Some(pair.1);
    }
  }
  return None;
}

