use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::boxed::Box;
use alloc::vec::Vec;

//use crate::SERIAL1;
//use alloc::format;

#[derive(Debug)]
pub enum FileSystemError {
  PathNotFound,
}

#[derive(Debug)]
pub enum FileSystemObject {
  File(String),
  Directory(Box<BTreeMap<String, FileSystemObject>>),
}

#[derive(Default)]
pub struct FileSystem {
  fs: BTreeMap<String, FileSystemObject>,
}

impl FileSystem {
  pub fn new() -> Self {
    Self {
      fs: BTreeMap::new(),
    }
  }

  fn create_at_path(&mut self, path: &Vec<String>, name: &str, content: Option<String>) -> Result<(), FileSystemError> {
    let mut current = &mut self.fs;
    for dir in path {
      current = match current.get_mut(dir).unwrap() {
        FileSystemObject::Directory(inside) => inside,
        _ => return Err(FileSystemError::PathNotFound),
      };
    }
    if let Some(content) = content {
      //file
      current.insert(name.to_string(), FileSystemObject::File(content));
    } else {
      //dir
      current.insert(name.to_string(), FileSystemObject::Directory(Box::new(BTreeMap::new())));
    }
    Ok(())
  }

  pub fn init(&mut self, contents: &[u8]) -> Result<(), FileSystemError> {
    let text = String::from_utf8_lossy(contents);
    //parse
    for file in text.split("@#@#@#[") {
      if file == String::new() {
        continue;
      }
      let mut split = file.split("]@#@#@#");
      let file_name = split.next().unwrap();
      let content = split.next().unwrap();
      let mut path = Vec::new();
      for level in file_name.split("/") {
        if level.contains(".") {
          self.create_at_path(&path, level, Some(content.to_string()))?;
          break;
        } else {
          self.create_at_path(&path, level, None)?;
          path.push(level.to_string());
        }
      }
    }
    //unsafe { SERIAL1.lock().write_text(&format!("{:?}", &self.fs)); }
    Ok(())
  }
}

