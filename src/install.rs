use crate::cache;
use crate::dpkg;
use crate::fetcher;
use crate::source::SourcePackage;
use colored::*;
use flate2::read::GzDecoder;
use glob;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path;
use users;
use xz2::read::XzDecoder;

pub fn do_install(package: &str) {
  let deb_regex = Regex::new(r"^.+\.deb$").unwrap();
  if deb_regex.is_match(package) {
    let debpath = path::Path::new(package);
    if !debpath.exists() {
      println!("No such file: {}", debpath.to_str().unwrap().to_string());
      return;
    }
    match install_deb(&debpath) {
      Ok(_) => {}
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    }
  } else {
    unimplemented!();
  }
}

pub fn install_deb(debfile: &path::Path) -> Result<(), String> {
  let tmp_workdir = path::Path::new("tmp");
  if !tmp_workdir.exists() {
    return Err("temporary working directory 'tmp' doesn't exist.".to_string());
  }

  // extract control.tar.gz
  let mut archive = ar::Archive::new(File::open(debfile).unwrap());
  let mut control_file_name = String::new();
  while let Some(entry_result) = archive.next_entry() {
    let mut entry = entry_result.unwrap();
    let _tmp = String::from(std::str::from_utf8(entry.header().identifier()).unwrap());
    if _tmp.contains("control") {
      control_file_name = _tmp;
    }
    let mut file = File::create(format!(
      "tmp/{}",
      std::str::from_utf8(entry.header().identifier()).unwrap()
    ))
    .unwrap();
    io::copy(&mut entry, &mut file).unwrap();
  }

  // extract control
  if control_file_name.contains(".gz") {
    let control_tar_gz = File::open(format!("tmp/{}", control_file_name)).unwrap();
    let control_tar = GzDecoder::new(control_tar_gz);
    let mut archive = tar::Archive::new(control_tar);
    archive.unpack("tmp").unwrap();
  } else if control_file_name.contains(".xz") {
    let control_tar_xz = File::open(format!("tmp/{}", control_file_name)).unwrap();
    let control_tar = XzDecoder::new(control_tar_xz);
    let mut archive = tar::Archive::new(control_tar);
    archive.unpack("tmp").unwrap();
  } else {
    return Err(format!(
      "Unknown control file archive format: {}",
      control_file_name
    ));
  }

  // read package info from control
  let control = std::fs::read_to_string("tmp/control").unwrap();
  let _packages = SourcePackage::from_row(&control).unwrap();
  let package = &cache::search_cache_with_name_glob(
    &glob::Pattern::new(&_packages.iter().nth(0).unwrap().package).unwrap(),
    true,
  )[0];
  let missing_package_names = match dpkg::get_missing_or_old_dependencies_recursive(package) {
    Ok(_missing_packages) => _missing_packages,
    Err(msg) => return Err(msg),
  };
  let missing_packages = cache::search_cache_with_names(&missing_package_names);

  print!("The following additional packages will be installed: \n  ");
  for mp in &missing_packages {
    print!("{} ", mp.package);
  }
  println!("");

  // XXX SHOULD distinguish between update and new install
  print!("The following NEW packages will be installed: \n  ");
  print!("{} ", package.package);
  for mp in &missing_packages {
    print!("{} ", mp.package);
  }
  println!("");

  println!(
    "{} upgraded, {} newly installed, {} to remove and {} not upgraded.",
    "?", "?", "?", "?"
  );
  println!("Need to get {} kB of archives.", "?");
  println!(
    "After this operation, {} MB of additional disk space will be used.",
    "?"
  );

  print!("Do you want to continue? [Y/n] ");
  std::io::stdout().flush();
  let mut user_yn = String::new();
  std::io::stdin()
    .read_line(&mut user_yn)
    .expect("invalid input");
  if user_yn != "y\n" && user_yn != "Y\n" {
    return Err("Abort.".to_string());
  }

  //// check permission
  //if users::get_current_uid() != 0 {
  //  return Err("install needs root permission.".to_string());
  //}

  // download all missing dependencies
  for (ix, md) in missing_packages.iter().enumerate() {
    print!("Get:{} {} ...", ix, md.package.green());
    std::io::stdout().flush();
    match fetcher::fetch_deb(&md) {
      Ok(_) => {
        println!("DONE");
      }
      Err(msg) => return Err(msg),
    }
    println!("Fetched {} kB in {}s ({} kB/s)", "?", "?", "?");
  }

  // install dependencies
  for md in missing_packages.iter().rev() {
    match dpkg::install_archived_package(&md) {
      Ok(()) => {},
      Err(msg) => return Err(msg),
    }
  }

  // install target 
  match dpkg::install_archived_package(&package) {
    Ok(()) => {},
    Err(msg) => return Err(msg),
  }

  Err("".to_string())
}
