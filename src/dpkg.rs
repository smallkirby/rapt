use crate::cache;
use crate::source::SourcePackage;
use crate::version::*;
use colored::*;
use indicatif::ProgressBar;
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

// XXX bug: 'libperl5.18' is regarded as missing when only 'libperl5.30' is installed.
pub fn check_missing_or_old(
  _package_name: &str,
  package_version: &Option<String>,
) -> Result<PackageState, String> {
  let installed_items = match read_dpkg_state() {
    Ok(_installed_items) => _installed_items,
    Err(msg) => return Err(msg),
  };

  let mut package_name = String::new();
  for c in _package_name.chars() {
    if c.is_digit(10) {
      break;
    } else {
      package_name.push(c);
    }
  }

  for ditem in installed_items {
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
            return Ok(PackageState::UPTODATE);
          } else {
            return Ok(PackageState::OLD);
          }
        }
        None => {
          return Ok(PackageState::UPTODATE); // if required version is missing, regard it as up-to-date
        }
      }
    }
  }

  Ok(PackageState::MISSING)
}

pub fn get_missing_or_old_dependencies(
  package: &SourcePackage,
) -> Result<Vec<(String, PackageState)>, String> {
  let mut ret_items = vec![];

  for (dep_package, dep_version) in &package.depends {
    match check_missing_or_old(dep_package, dep_version) {
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
  acc: &Vec<(String, PackageState)>,
) -> Result<Vec<(String, PackageState)>, String> {
  let mut ret_items: Vec<(String, PackageState)> = acc.clone();

  // search missing/old dependencies for @package
  let mut missing_package_names = match get_missing_or_old_dependencies(package) {
    Ok(_missing_package_name) => _missing_package_name,
    Err(msg) => return Err(msg),
  };

  // get instances of missing/old packages
  let missing_packages = cache::search_cache_with_names(
    &missing_package_names
      .iter()
      .map(|p| p.0.clone())
      .collect::<Vec<_>>(),
  );
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

  ret_items.append(&mut missing_package_names);
  // recursively search missing/old dependencies
  for p in missing_packages {
    match sub_missing_or_old_dependencies_recursive(&p, acc) {
      Ok(mut names) => {
        ret_items.append(&mut names);
      }
      Err(msg) => return Err(msg),
    }
  }

  Ok(ret_items)
}

pub fn get_missing_or_old_dependencies_recursive(
  package: &SourcePackage,
) -> Result<Vec<(String, PackageState)>, String> {
  sub_missing_or_old_dependencies_recursive(package, &mut vec![])
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
