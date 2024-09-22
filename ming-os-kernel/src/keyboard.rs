
#[derive(Debug)]
pub enum KeyChar {
  Press(char),
  SpecialPress(&'static str),
  SpecialRelease(&'static str),
}

//https://wiki.osdev.org/PS/2_Keyboard#Scan_Code_Set_1
pub fn scancode_to_char(scancode: u8) -> Option<KeyChar> {
  match scancode {
    0x02 => Some(KeyChar::Press('1')),
    0x03 => Some(KeyChar::Press('2')),
    0x04 => Some(KeyChar::Press('3')),
    0x05 => Some(KeyChar::Press('4')),
    0x06 => Some(KeyChar::Press('5')),
    0x07 => Some(KeyChar::Press('6')),
    0x08 => Some(KeyChar::Press('7')),
    0x09 => Some(KeyChar::Press('8')),
    0x0A => Some(KeyChar::Press('9')),
    0x0B => Some(KeyChar::Press('0')),
    //
    0x10 => Some(KeyChar::Press('q')),
    0x11 => Some(KeyChar::Press('w')),
    0x12 => Some(KeyChar::Press('e')),
    0x13 => Some(KeyChar::Press('r')),
    0x14 => Some(KeyChar::Press('t')),
    0x15 => Some(KeyChar::Press('y')),
    0x16 => Some(KeyChar::Press('u')),
    0x17 => Some(KeyChar::Press('i')),
    0x18 => Some(KeyChar::Press('o')),
    0x19 => Some(KeyChar::Press('p')),
    //
    0x1E => Some(KeyChar::Press('a')),
    0x1F => Some(KeyChar::Press('s')),
    0x20 => Some(KeyChar::Press('d')),
    0x21 => Some(KeyChar::Press('f')),
    0x22 => Some(KeyChar::Press('g')),
    0x23 => Some(KeyChar::Press('h')),
    0x24 => Some(KeyChar::Press('j')),
    0x25 => Some(KeyChar::Press('k')),
    0x26 => Some(KeyChar::Press('l')),
    //
    0x2C => Some(KeyChar::Press('z')),
    0x2D => Some(KeyChar::Press('x')),
    0x2E => Some(KeyChar::Press('c')),
    0x2F => Some(KeyChar::Press('v')),
    0x30 => Some(KeyChar::Press('b')),
    0x31 => Some(KeyChar::Press('n')),
    0x32 => Some(KeyChar::Press('m')),
    //
    0x38 => Some(KeyChar::SpecialPress("alt")),
    //
    0xB8 => Some(KeyChar::SpecialRelease("alt")),
    _ => None,
  }
}

