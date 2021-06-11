use colored::*;
use file_lock;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LockType {
  DIR,
  FILE,
}

#[derive(Debug, PartialEq)]
pub enum Lock {
  ARCHIVE(LockType),
  LIST(LockType),
}

pub fn get_lock(lock: Lock) -> Result<(), String> {
  match lock {
    Lock::ARCHIVE(t) => match t {
      LockType::DIR => {}
      LockType::FILE => {}
    },
    Lock::LIST(t) => match t {
      LockType::DIR => match setup_lock_dir("lists", "partial", 0o700, true) {
        Ok(()) => {}
        Err(_) => return Err("Failed to get a lock.".to_string()),
      },
      LockType::FILE => {}
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
