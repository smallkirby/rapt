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
    let found_items = filter_package_with_name(&package_glob, &installed_items);
    list_packages(&found_items);
  } else if upgradable {
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(
      ProgressStyle::default_bar().template("Checking dpkg status       : {bar:40} {msg}"),
    );

    let cached_items = &*source::CACHE;
    let upgradable_items = match dpkg::check_upgradable(&cached_items, Some(&progress_bar)) {
      Ok(_upgradable_items) => _upgradable_items,
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    };
    list_packages(&upgradable_items);
  } else {
    let package_glob = match Pattern::new(package) {
      Ok(_package_glob) => _package_glob,
      Err(_) => {
        println!("invalid glob pattern: {}", package);
        return;
      }
    };
    let found_items = cache::search_cache_with_name_glob(&package_glob, false);
    list_packages(&found_items);
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
    // maybe, should search apt/lists/** for them.
    let arch = match item.arch.iter().nth(0) {
      Some(_tmp) => _tmp.to_string(),
      None => "".to_string(),
    };
    println!(
      "{}/{} {} {}",
      item.package.green().bold(),
      item.component,
      item.version,
      arch,
    );
  }
}
