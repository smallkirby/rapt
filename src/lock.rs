use colored::*;
use file_lock;
use file_lock::FileLock;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Lock {
  ARCHIVE,
  LIST,
}

pub fn get_lock(lock: Lock) -> Result<FileLock, String> {
  let lockdir_name = match lock {
    Lock::ARCHIVE => "archive",
    Lock::LIST => "lists",
  };
  match setup_lock_dir(lockdir_name, "partial", 0o700, true) {
    Ok(()) => {}
    Err(_) => return Err("Failed to get a lock.".to_string()),
  }
  match setup_lock_dir(lockdir_name, "auxfiles", 0o755, true) {
    Ok(()) => {}
    Err(_) => return Err("Failed to get a lock.".to_string()),
  }

  do_get_lock(&format!("{}/lock", lockdir_name))
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

pub fn do_get_lock(lockpath: &str) -> Result<FileLock, String> {
  match FileLock::lock(lockpath, false, true) {
    Ok(lock) => Ok(lock),
    Err(_) => Err(format!("Failed to get a lock: {}", lockpath.red().bold()).to_string()),
  }
}
