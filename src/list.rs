use crate::source::SourcePackage;
use crate::{cache, dpkg, source};
use colored::*;
use glob::Pattern;
use indicatif::{ProgressBar, ProgressStyle};

pub fn do_list(package: &str, installed: bool, upgradable: bool) {
  if installed {
    let installed_items = &*source::DPKG_CACHE;
    // 'apt list' uses glob pattern instead of regex.
    let package_glob = match Pattern::new(package) {
      Ok(_r) => _r,
      Err(_) => {
        println!("invalid glob pattern: {}", package);
        return;
      }
    };
    let mut found_items = filter_package_with_name(&package_glob, &installed_items);
    list_packages(&mut found_items);
  } else if upgradable {
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(
      ProgressStyle::default_bar().template("Checking dpkg status       : {bar:40} {msg}"),
    );

    let cached_items = &*source::CACHE;
    let mut upgradable_items = match dpkg::check_upgradable(&cached_items, Some(&progress_bar)) {
      Ok(_upgradable_items) => _upgradable_items,
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    };
    list_packages(&mut upgradable_items);
  } else {
    let package_glob = match Pattern::new(package) {
      Ok(_package_glob) => _package_glob,
      Err(_) => {
        println!("invalid glob pattern: {}", package);
        return;
      }
    };
    let mut found_items = cache::search_cache_with_name_glob(&package_glob, false);
    list_packages(&mut found_items);
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

pub fn list_packages(items: &mut Vec<SourcePackage>) {
  for item in items {
    // first, check if it is installed and auto-installed
    let auto_installed = match source::EXTENDED_CACHE.iter().find(|e| e.0 == item.package) {
      Some(info) => info.1,
      None => false,
    };
    let installed = if cache::search_cache_with_name_glob(
      &glob::Pattern::new(&item.package).unwrap(),
      true,
    )
    .len()
      >= 1
    {
      true
    } else {
      false
    };

    // show lists
    let arch = match item.arch.iter().nth(0) {
      Some(_tmp) => _tmp.to_string(),
      None => "".to_string(),
    };
    print!(
      "{}/{} {} {} ",
      item.package.green().bold(),
      item.component,
      item.version,
      arch,
    );
    if installed && auto_installed {
      println!("[installed,automatic]");
    } else if installed {
      println!("[installed]");
    } else if auto_installed {
      println!("[this package might be broken.]");
    } else {
      println!("");
    }
  }
}
