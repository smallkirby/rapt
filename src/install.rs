use crate::cache;
use crate::dpkg;
use crate::fetcher;
use crate::source::SourcePackage;
use colored::*;
use flate2::read::GzDecoder;
use glob;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path;
use std::sync::mpsc;
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
    // search package information from cache
    let _target_package =
      &cache::search_cache_with_name_glob(&glob::Pattern::new(package).unwrap(), true);
    if _target_package.len() == 0 {
      println!(
        "Package {} is not in cache. \nDo 'rapt update' or add sources.list.",
        package.green()
      );
      return;
    }
    // check the package status
    let target_package = &_target_package[0];
    let progress_bar = ProgressBar::new(0);
    progress_bar
      .set_style(ProgressStyle::default_bar().template("Checking dependencies: {bar:40} {msg}"));
    match dpkg::check_missing_or_old(
      &target_package.package,
      &Some(target_package.version.clone()),
      Some(&progress_bar),
      None,
    )
    .unwrap()
    {
      dpkg::PackageState::MISSING => match install_package(&target_package) {
        Ok(()) => {}
        Err(msg) => {
          println!("{}", msg);
          return;
        }
      },
      dpkg::PackageState::UPTODATE | dpkg::PackageState::OLD => {
        println!(
          "Package {} is already installed.",
          target_package.package.green()
        );
      }
    }
  }
}

pub fn install_package(package: &SourcePackage) -> Result<(), String> {
  // install target package's deb
  let progress_bar = ProgressBar::new(0);
  progress_bar.set_style(
    ProgressStyle::default_bar()
      .template("Get: [{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
      .progress_chars("#>-"),
  );
  let debname = match fetcher::fetch_deb(&package, Some(&progress_bar)) {
    Ok(_debname) => _debname.0,
    Err(msg) => return Err(msg),
  };
  println!("fetched {} into {}", package.package, debname);

  install_deb(&path::Path::new(&format!("archive/{}", debname)))
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

  // find missing/old dependencies
  let package = &cache::search_cache_with_name_glob(
    &glob::Pattern::new(&_packages.iter().nth(0).unwrap().package).unwrap(),
    true,
  )[0];
  println!("\nRecursively searching for dependencies: ");
  let missing_old_package_names = dpkg::get_missing_or_old_dependencies_recursive(package, true)?;
  let missing_package_names = missing_old_package_names
    .iter()
    .filter(|c| c.1 == dpkg::PackageState::MISSING)
    .collect::<Vec<_>>();
  let old_package_names = missing_old_package_names
    .iter()
    .filter(|c| c.1 == dpkg::PackageState::OLD)
    .collect::<Vec<_>>();
  let missing_packages = cache::search_cache_with_names(
    &missing_package_names
      .iter()
      .map(|item| item.0.clone())
      .collect(),
  );
  let old_packages = cache::search_cache_with_names(
    &old_package_names
      .iter()
      .map(|item| item.0.clone())
      .collect(),
  );

  print!("\nThe following additional packages will be installed: \n  ");
  for mp in &missing_packages {
    print!("{} ", mp.package);
  }
  for mp in &old_packages {
    print!("{} ", mp.package);
  }
  println!("");

  print!("The following NEW packages will be installed: \n  ");
  print!("{} ", package.package.green().bold());
  for mp in &missing_packages {
    print!("{} ", mp.package.green());
  }
  println!("");

  println!(
    "{} upgraded, {} newly installed, {} to remove and {} not upgraded.",
    old_packages.len(),
    missing_packages.len() + 1,
    "?",
    "?"
  );
  println!("Need to get {} kB of archives.", "?");
  println!(
    "After this operation, {} MB of additional disk space will be used.",
    "?"
  );

  print!("Do you want to continue? [Y/n] ");
  std::io::stdout().flush().unwrap();
  let mut user_yn = String::new();
  std::io::stdin()
    .read_line(&mut user_yn)
    .expect("invalid input");
  if user_yn != "y\n" && user_yn != "Y\n" {
    return Err("Abort.".to_string());
  }

  // XXX
  //// check permission
  //if users::get_current_uid() != 0 {
  //  return Err("install needs root permission.".to_string());
  //}

  // download all missing dependencies
  let mut handles = vec![];
  let (tx, rx) = mpsc::channel();
  let progress_bars = MultiProgress::new();
  let progress_style = ProgressStyle::default_bar()
    .template("Get: [{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
    .progress_chars("#>-");

  for (_ix, _md) in missing_packages
    .iter()
    .chain(old_packages.iter())
    .enumerate()
  {
    let md = _md.clone();
    let tx = tx.clone();
    let progress_bar = progress_bars.add(ProgressBar::new(999999999));
    progress_bar.set_style(progress_style.clone());

    let handle = std::thread::spawn(move || match fetcher::fetch_deb(&md, Some(&progress_bar)) {
      Ok((_filename, fetched_size)) => {
        tx.send(Ok(fetched_size)).unwrap();
      }
      Err(msg) => {
        tx.send(Err(msg)).unwrap();
      }
    });
    handles.push(handle);
  }

  let mut fetched_amount = 0;
  progress_bars.join().unwrap();
  for handle in handles {
    match rx.recv().unwrap() {
      Ok(fetched_size) => {
        fetched_amount += fetched_size;
      }
      Err(msg) => {
        println!("{}", msg);
        return Err(msg);
      }
    }
    handle.join().unwrap();
  }
  println!("Fetched {} kB in ?s (? kB/s)", fetched_amount / 1000);

  // install dependencies
  for (_ix, md) in missing_packages
    .iter()
    .chain(old_packages.iter())
    .rev()
    .enumerate()
  {
    println!("installing {} ...", md.package.green());
    dpkg::install_archived_package(&md)?;
  }

  // install target
  println!("installing {} ...", package.package.green().bold());
  dpkg::install_archived_package(&package)?;

  println!("{}", "Install complete.".yellow().bold());

  Err("".to_string())
}

#[cfg(test)]
pub mod test {
  #[allow(dead_code)]
  fn test_vim_tiny() {
    let package = "vim-common";
    let items =
      crate::cache::search_cache_with_name_glob(&glob::Pattern::new(package).unwrap(), true);
    let missing = crate::dpkg::get_missing_or_old_dependencies(&items[0], true);
    println!("{:?}", missing);
    println!(
      "{:?}",
      crate::dpkg::check_missing_or_old("xxd", &None, None, None)
    );
    panic!("");
  }
}
