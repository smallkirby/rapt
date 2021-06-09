use crate::cache;
use crate::source::SourcePackage;
use crate::version::*;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PackageState {
  MISSING,
  OLD,
  UPTODATE,
}

pub fn read_dpkg_state() -> Result<Vec<SourcePackage>, String> {
  let raw_packages = match std::fs::read_to_string("/var/lib/dpkg/status") {
    Ok(_raw_packages) => _raw_packages,
    Err(_msg) => return Err(_msg.to_string()),
  };

  SourcePackage::from_row(&raw_packages)
}

pub fn check_upgradable(
  index_items: &Vec<SourcePackage>,
  _progress_bar: Option<&ProgressBar>,
) -> Result<Vec<SourcePackage>, String> {
  let mut upgradable_items = vec![];

  let dpkg_items = match read_dpkg_state() {
    Ok(_dpkg_items) => _dpkg_items,
    Err(msg) => return Err(msg),
  };
  if _progress_bar.is_some() {
    _progress_bar.unwrap().set_length(dpkg_items.len() as u64);
    _progress_bar.unwrap().set_position(0);
  }

  for ditem in dpkg_items {
    if _progress_bar.is_some() {
      _progress_bar.unwrap().set_message(ditem.package.clone());
      _progress_bar.unwrap().inc(1);
    }
    let iitems = index_items
      .iter()
      .filter(|item| item.package == ditem.package)
      .collect::<Vec<_>>();
    if iitems.len() == 0 {
      log::warn!(
        "package is missing, but ignoring for now: {}",
        ditem.package
      );
      continue;
    }
    let iitem = iitems[0];

    let cmp_res = comp_version(&iitem.version, &ditem.version);
    if cmp_res > 0 {
      upgradable_items.push(ditem);
    }
  }

  if _progress_bar.is_some() {
    _progress_bar.unwrap().finish_with_message("DONE");
  }
  Ok(upgradable_items)
}

pub fn check_missing_or_old(
  _package_name: &str,
  package_version: &Option<String>,
  _progress_bar: Option<&ProgressBar>,
) -> Result<PackageState, String> {
  let installed_items = match read_dpkg_state() {
    Ok(_installed_items) => _installed_items,
    Err(msg) => return Err(msg),
  };
  let finalize_progress_bar = {
    || {
      if _progress_bar.is_some() {
        _progress_bar.unwrap().finish_with_message("DONE");
      }
    }
  };
  if _progress_bar.is_some() {
    _progress_bar
      .unwrap()
      .set_length(installed_items.len() as u64);
    _progress_bar.unwrap().set_position(0);
  }

  let package_name = String::from(_package_name);

  let total_installed_item = installed_items.len();
  for (ix, ditem) in installed_items.iter().enumerate() {
    if _progress_bar.is_some() {
      _progress_bar.unwrap().set_message(format!(
        "{:>3}% : {}",
        (ix as f32 / total_installed_item as f32) * 100 as f32,
        ditem.package
      ));
      _progress_bar.unwrap().inc(1);
    }
    let mut dpackage_name = String::new();
    for c in ditem.package.chars() {
      if c.is_digit(10) {
        break;
      } else {
        dpackage_name.push(c);
      }
    }
    if dpackage_name == package_name {
      match package_version {
        Some(v) => {
          let res_cmp_version = comp_version(&ditem.version, &v);
          if res_cmp_version >= 0 {
            finalize_progress_bar();
            return Ok(PackageState::UPTODATE);
          } else {
            finalize_progress_bar();
            return Ok(PackageState::OLD);
          }
        }
        None => {
          finalize_progress_bar();
          return Ok(PackageState::UPTODATE); // if required version is missing, regard it as up-to-date
        }
      }
    }
  }

  finalize_progress_bar();
  Ok(PackageState::MISSING)
}

pub fn get_missing_or_old_dependencies(
  package: &SourcePackage,
  show_progress: bool,
) -> Result<Vec<(String, PackageState)>, String> {
  let mut ret_items = vec![];

  for (dep_package, dep_version) in &package.depends {
    let tmp = ProgressBar::new(0);
    tmp.set_style(
      ProgressStyle::default_bar().template(&format!("Check deps {}: {{msg}}", dep_package)),
    );
    let progress_bar = if show_progress { Some(&tmp) } else { None };

    match check_missing_or_old(dep_package, dep_version, progress_bar) {
      Ok(is_missing) => match is_missing {
        PackageState::MISSING => ret_items.push((dep_package.clone(), PackageState::MISSING)),
        PackageState::OLD => ret_items.push((dep_package.clone(), PackageState::OLD)),
        _ => {}
      },
      Err(msg) => return Err(msg),
    }
  }

  Ok(ret_items)
}

fn sub_missing_or_old_dependencies_recursive(
  package: &SourcePackage,
  acc: &mut Vec<(String, PackageState)>,
  //progress_style: Option<&ProgressStyle>,
  show_progress: bool,
) -> Result<Vec<(String, PackageState)>, String> {
  // search missing/old dependencies for @package
  let mut missing_package_names = match get_missing_or_old_dependencies(package, show_progress) {
    Ok(_missing_package_name) => _missing_package_name,
    Err(msg) => return Err(msg),
  };

  // get instances of missing/old packages
  let mut missing_packages = cache::search_cache_with_names(
    &missing_package_names
      .iter()
      .map(|p| p.0.clone())
      .collect::<Vec<_>>(),
  );

  // remove duplicated packages
  missing_packages = missing_packages
    .into_iter()
    .filter(|item| {
      acc
        .iter()
        .find(|a| a.0 == item.package || item.package == package.package)
        .is_none()
    })
    .collect::<Vec<_>>();
  missing_package_names = missing_package_names
    .into_iter()
    .filter(|name| {
      acc
        .iter()
        .find(|a| a.0 == name.0 || name.0 == package.package)
        .is_none()
    })
    .collect::<Vec<_>>();

  // return error if some dependency is not in cache.
  if missing_packages.len() != missing_package_names.len() {
    let mut diffs = String::new();
    for n in missing_package_names {
      let mut found = false;
      for p in &missing_packages {
        if p.package == n.0 {
          found = true;
          break;
        }
      }
      if !found {
        diffs.push_str(&format!("{} ", n.0));
      }
    }
    return Err(format!(
      "Dependency packages not found in cache files: {}",
      diffs
    ));
  }

  acc.append(&mut missing_package_names);
  // recursively search missing/old dependencies
  for p in missing_packages {
    match sub_missing_or_old_dependencies_recursive(&p, acc, show_progress) {
      Ok(mut names) => {
        names = names
          .into_iter()
          .filter(|item| acc.iter().find(|a| a.0 == item.0).is_none())
          .collect::<Vec<_>>();
        acc.append(&mut names);
      }
      Err(msg) => return Err(msg),
    }
  }

  Ok(acc.to_vec())
}

pub fn get_missing_or_old_dependencies_recursive(
  package: &SourcePackage,
  show_progress: bool,
) -> Result<Vec<(String, PackageState)>, String> {
  sub_missing_or_old_dependencies_recursive(package, &mut vec![], show_progress)
}

pub fn install_archived_package(package: &SourcePackage) -> Result<(), String> {
  let _a = package.filename.rfind('/').unwrap();
  let debname = format!("archive/{}", &package.filename[_a + 1..]);

  let output = Command::new("dpkg")
    .args(&["-i", &debname])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap()
    .wait_with_output()
    .unwrap();
  let outstr = String::from_utf8(output.stdout).unwrap();
  println!("{}", outstr);

  if !output.status.success() {
    let errstr = String::from_utf8(output.stderr).unwrap();
    println!("{}", errstr.red());
    return Err("dpkg exited with failing error code.".to_string());
  }

  Ok(())
}
