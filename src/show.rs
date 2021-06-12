use crate::source::SourcePackage;
use crate::{dpkg, source};
use colored::*;
use glob::Pattern;

// XXX for now, support only single glob term
pub fn do_show(package: &str) {
  let package_glob = match Pattern::new(package) {
    Ok(_package_glob) => _package_glob,
    Err(_) => {
      println!("invalid glob pattern: {}", package);
      return;
    }
  };
  let found_items = dpkg::search_dpkg_with_name_glob(&package_glob, false);
  let found_resolved_items = source::resolve_duplication(&found_items, None).unwrap();

  println!("");
  list_packages(&found_resolved_items);
}

pub fn list_packages(items: &Vec<SourcePackage>) {
  // XXX should show distro/arch, but dpkg/status doesn't have these info.
  // maybe, should show apt/lists/** for them.
  for item in items {
    println!("Package: {}", item.package.green().bold());
    println!("Version: {}", item.version);
    println!("Priority: {}", item.priority.to_string().to_lowercase());
    println!("Section: {}", item.section.to_string().to_lowercase());
    println!("Origin: {}", item.origin);
    println!("Maintainer: {}", item.maintainer);
    println!("Original-Maintainer: {}", item.original_maintainer);
    println!("Bugs: {}", item.bugs);
    print!("Depends: ");
    for dep in &item.depends {
      match dep.1 {
        Some(version) => {
          print!("{} (>= {}), ", dep.0, version);
        }
        None => {
          print!("{}, ", dep.0);
        }
      }
    }
    println!("");
    print!("Suggests: ");
    for pre in &item.suggests {
      print!("{}, ", pre);
    }
    println!("");
    println!("Homepage: {}", item.homepage);
    println!("Description: {}", item.description);
    println!("");
  }
}
