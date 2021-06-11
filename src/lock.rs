use colored::*;
use file_lock;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LOCK_TYPE {
  DIR,
  FILE,
}

#[derive(Debug, PartialEq)]
pub enum LOCK {
  ARCHIVE(LOCK_TYPE),
  LIST(LOCK_TYPE),
}

pub fn get_lock(lock: LOCK) -> Result<(), String> {
  match lock {
    LOCK::ARCHIVE(t) => match t {
      LOCK_TYPE::DIR => {}
      LOCK_TYPE::FILE => {}
    },
    LOCK::LIST(t) => match t {
      LOCK_TYPE::DIR => match setup_lock_dir("lists", "partial", 0o700, true) {
        Ok(()) => {}
        Err(_) => return Err("Failed to get a lock.".to_string()),
      },
      LOCK_TYPE::FILE => {}
    },
  };

  Ok(())
}

fn setup_lock_dir(dirname: &str, postfix: &str, mode: u32, is_dir: bool) -> Result<(), String> {
  let target = format!("{}/{}", dirname, postfix);
  if !Path::new(&target).exists() {
    if is_dir {
      match std::fs::create_dir(&target) {
        Ok(_) => println!("Created lock dir: {}", target.blue()),
        Err(_) => {
          return Err(format!(
            "{} List directory {} is missing, and can't create.",
            "E:".red().bold(),
            target
          ))
        }
      }
    } else {
      match std::fs::File::create(&target) {
        Ok(_) => println!("Created lock file: {}", target.blue()),
        Err(_) => {
          return Err(format!(
            "{} List file {} is missing, and can't create.",
            "E:".red().bold(),
            target
          ))
        }
      }
    }
  }

  let meta = std::fs::File::open(&target).unwrap().metadata().unwrap();
  let mut perm = meta.permissions();
  perm.set_mode(mode);

  Ok(())
}
