use crate::cache;
use crate::dpkg;
use crate::source::SourcePackage;
use colored::*;
use regex;

// XXX for now, support only single regex term
pub fn do_search(package: &str, show_full: bool) {
  let package_regex = match regex::Regex::new(package) {
    Ok(_package_regex) => _package_regex,
    Err(_) => {
      println!("invalid regex pattern: {}", package);
      return;
    }
  };
  let found_items = cache::search_cache_with_name_description_regex(&package_regex, false);
  list_packages(&found_items, show_full);
}

pub fn list_packages(items: &Vec<SourcePackage>, show_full: bool) {
  // XXX should show distro/arch, but dpkg/status doesn't have these info.
  // maybe, should search apt/lists/** for them.
  for item in items {
    println!(
      "{}/{} {} {}",
      item.package.green().bold(),
      "?",
      item.version,
      "?"
    );
    if show_full {
      println!(
        "  {}\n",
        item
          .description
          .split("\n")
          .collect::<Vec<_>>()
          .iter()
          .nth(0)
          .unwrap()
      );
    } else {
      for s in item.description.split("\n").collect::<Vec<_>>() {
        println!("  {}", s);
      }
      println!("");
    }
  }
}
