use crate::dpkg;
use crate::source::SourcePackage;
use colored::*;
use glob::Pattern;

pub fn do_list(package: &str, installed: bool) {
  if installed {
    let installed_items = match dpkg::read_dpkg_state() {
      Ok(_installed_items) => _installed_items,
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    };
    // 'apt list' uses glob pattern instead of regex.
    let package_glob = match Pattern::new(package) {
      Ok(_r) => _r,
      Err(_) => {
        println!("invalid glob pattern: {}", package);
        return;
      }
    };
    let found_items = filter_package_with_name(&package_glob, &installed_items);
    list_packages(&found_items);
  } else {
    unimplemented!();
  }
}

pub fn filter_package_with_name(glob: &Pattern, items: &Vec<SourcePackage>) -> Vec<SourcePackage> {
  let mut found_items = vec![];
  for item in items {
    if glob.matches(&item.package) {
      found_items.push(item.clone());
    }
  }

  found_items
}

pub fn list_packages(items: &Vec<SourcePackage>) {
  for item in items {
    // XXX should show distro/arch, but dpkg/status doesn't have these info.
    // maybe search apt/lists/** for them.
    println!(
      "{}/{} {} {}",
      item.package.green().bold(),
      "?",
      item.version,
      "?"
    );
  }
}
