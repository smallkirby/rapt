use crate::lock::{get_lock, Lock};
use crate::slist;
use crate::source::{self, SourcePackage, CACHE};
use glob::Pattern;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn get_info_from_filename(filename: &str) -> (String, String) {
  let tmp = filename.split("_dists_").collect::<Vec<_>>();
  let part = match tmp.iter().nth(1) {
    Some(_part) => _part,
    None => return ("".to_string(), "".to_string()),
  };
  let ix = match part.rfind("-") {
    Some(_ix) => _ix,
    None => return ("".to_string(), "".to_string()),
  };
  let dist = String::from(&part[..ix]);
  let component = String::from(&part[ix + 1..]);

  (dist, component)
}

pub fn get_pool_domain(package: &SourcePackage) -> Result<String, ()> {
  match glob::glob("lists/*") {
    Ok(paths) => {
      for entry in paths {
        match entry {
          Ok(path) => {
            if path.is_dir() || path.file_name().unwrap().to_str().unwrap() == "lock" {
              continue;
            }
            let raw_cache = match std::fs::read_to_string(&path) {
              Ok(_raw_cache) => _raw_cache,
              Err(msg) => {
                println!("{}", msg);
                return Err(());
              }
            };
            match source::SourcePackage::from_raw(
              &raw_cache,
              path.file_name().unwrap().to_str().unwrap(),
            ) {
              Ok(_items) => {
                if _items
                  .iter()
                  .filter(|i| i.package == package.package)
                  .collect::<Vec<_>>()
                  .len()
                  != 0
                {
                  let mut domain = String::new();
                  let filename = String::from(path.file_name().unwrap().to_str().unwrap());
                  let mut count = 0;
                  for c in filename.chars() {
                    if c == '_' {
                      domain.push('/');
                      count += 1;
                    } else {
                      domain.push(c);
                    }
                    if count == 2 {
                      break;
                    }
                  }
                  return Ok(domain);
                }
              }
              Err(msg) => {
                println!("{}", msg);
                return Err(());
              }
            };
          }
          Err(msg) => {
            println!("failed to open cache file: {}", msg);
            return Err(());
          }
        };
      }
    }
    Err(_) => {
      println!("invalid glob pattern.");
      return Err(());
    }
  };
  return Err(());
}

pub fn search_cache_with_names(names: &Vec<String>) -> Vec<SourcePackage> {
  let mut ret_items = vec![];
  for name in names {
    let name_glob = glob::Pattern::new(name.split(":").collect::<Vec<_>>()[0]).unwrap();
    let mut founds = search_cache_with_name_glob(&name_glob, true);
    ret_items.append(&mut founds);
  }

  ret_items
}

// @cache should be resolved in duplication.
// XXX for now, return items whose 'Provides' matches.
pub fn search_cache_with_name_glob(glob: &Pattern, case_sensitive: bool) -> Vec<SourcePackage> {
  let mut ret_items = vec![];
  let cached_items = &*CACHE;

  for item in cached_items {
    if case_sensitive {
      if glob.matches(&item.package) || item.provides.iter().any(|d| glob.matches(d)) {
        ret_items.push(item.clone());
      }
    } else {
      if glob.matches(&item.package.to_lowercase())
        || item
          .provides
          .iter()
          .any(|d| glob.matches(&d.to_lowercase()))
      {
        ret_items.push(item.clone());
      }
    }
  }

  ret_items
}

pub fn search_cache_with_name_description_regex(
  reg: &regex::Regex,
  case_sensitive: bool,
) -> Vec<SourcePackage> {
  let mut ret_items = vec![];
  let cached_items = get_cached_items();
  for item in cached_items {
    if case_sensitive {
      if reg.is_match(&item.package) {
        ret_items.push(item.clone());
      } else if reg.is_match(&item.description) {
        ret_items.push(item.clone());
      }
    } else {
      if reg.is_match(&item.package.to_lowercase()) {
        ret_items.push(item.clone());
      } else if reg.is_match(&item.description.to_lowercase()) {
        ret_items.push(item.clone());
      }
    }
  }

  ret_items
}

pub fn get_cached_items() -> Vec<SourcePackage> {
  let mut ret_items = vec![];
  let lock = match get_lock(Lock::LIST) {
    Ok(_lock) => _lock,
    Err(_) => {
      println!("Failed to get a lock: lists/lock");
      return vec![];
    }
  };

  match glob::glob("lists/*") {
    Ok(paths) => {
      for entry in paths {
        match entry {
          Ok(path) => {
            if path.is_dir() {
              continue;
            }
            let filename = path.file_name().unwrap();
            if filename == "lock" {
              continue;
            }
            let raw_cache = match std::fs::read_to_string(&path) {
              Ok(_raw_cache) => _raw_cache,
              Err(msg) => {
                lock.unlock().unwrap();
                println!("{}", msg);
                return vec![];
              }
            };
            match source::SourcePackage::from_raw(&raw_cache, filename.to_str().unwrap()) {
              Ok(mut _items) => ret_items.append(&mut _items),
              Err(msg) => {
                lock.unlock().unwrap();
                println!("{}", msg);
                return ret_items;
              }
            };
          }
          Err(msg) => {
            lock.unlock().unwrap();
            println!("failed to open cache file: {}", msg);
            return vec![];
          }
        };
      }
    }
    Err(_) => {
      lock.unlock().unwrap();
      println!("invalid glob pattern.");
      return vec![];
    }
  };

  lock.unlock().unwrap();
  ret_items
}

pub fn write_cache_raw(raw_index: &str, source: &slist::Source) -> Result<(), String> {
  let filename = source.to_filename();
  if !Path::new("lists").exists() {
    return Err("cache directory 'lists' doesn't exist. aborting...".to_string());
  };
  if Path::new(&format!("lists/{}", filename)).exists() {
    // clean the file for simplicity
    fs::remove_file(format!("lists/{}", filename)).unwrap();
  };

  log::info!("creating cache file: {}", format!("lists/{}", filename));
  let mut out = fs::File::create(format!("lists/{}", filename)).unwrap();
  write!(out, "{}", raw_index).unwrap();

  Ok(())
}
