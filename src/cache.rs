use crate::slist;
use crate::source;
use crate::source::SourcePackage;
use glob::Pattern;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn get_pool_domain(package: &SourcePackage) -> Result<String, ()> {
  match glob::glob("lists/*") {
    Ok(paths) => {
      for entry in paths {
        match entry {
          Ok(path) => {
            let raw_cache = match std::fs::read_to_string(&path) {
              Ok(_raw_cache) => _raw_cache,
              Err(msg) => {
                println!("{}", msg);
                return Err(());
              }
            };
            match source::SourcePackage::from_row(&raw_cache) {
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
    let name_glob = glob::Pattern::new(&name).unwrap();
    let mut founds = search_cache_with_name_glob(&name_glob, true);
    ret_items.append(&mut founds);
  }

  ret_items
}

pub fn search_cache_with_name_glob(glob: &Pattern, case_sensitive: bool) -> Vec<SourcePackage> {
  let mut ret_items = vec![];
  let cached_items = get_cached_items();
  for item in cached_items {
    if case_sensitive {
      if glob.matches(&item.package) {
        ret_items.push(item.clone());
      }
    } else {
      if glob.matches(&item.package.to_lowercase()) {
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

  match glob::glob("lists/*") {
    Ok(paths) => {
      for entry in paths {
        match entry {
          Ok(path) => {
            let raw_cache = match std::fs::read_to_string(path) {
              Ok(_raw_cache) => _raw_cache,
              Err(msg) => {
                println!("{}", msg);
                return vec![];
              }
            };
            match source::SourcePackage::from_row(&raw_cache) {
              Ok(mut _items) => ret_items.append(&mut _items),
              Err(msg) => {
                println!("{}", msg);
                return ret_items;
              }
            };
          }
          Err(msg) => {
            println!("failed to open cache file: {}", msg);
            return vec![];
          }
        };
      }
    }
    Err(_) => {
      println!("invalid glob pattern.");
      return vec![];
    }
  };

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
