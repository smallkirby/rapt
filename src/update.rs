use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::mpsc;
use std::thread;

use crate::source::SourcePackage;
use crate::{cache, dpkg, fetcher, lock, slist, source};

pub fn do_update() {
  log::trace!("do_update()");

  let mut package_items = vec![];

  // read sources.list
  let sources = match slist::parse_source_file("sources.list") {
    Ok(_items) => _items,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  let lock = match lock::get_lock(lock::Lock::LIST) {
    Ok(_lock) => _lock,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  let start_time = std::time::SystemTime::now();
  // fetch index files and get package items.
  println!("Fetching indexes... ");

  let mut fetched_amount = 0;
  match fetch_indexes_thread(&sources) {
    Ok((fetched_sizes, mut items)) => {
      for s in fetched_sizes {
        fetched_amount += s;
      }
      package_items.append(&mut items);
    }
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  }
  let total_time = start_time.elapsed().unwrap().as_secs();
  let fetched_amount_kb: u64 = (fetched_amount / 1024).into();
  let bps = if total_time == 0 {
    fetched_amount_kb
  } else {
    fetched_amount_kb / total_time
  };
  println!(
    "Fetched {} kB in {}s ({} kB/s)",
    fetched_amount_kb, total_time, bps
  );

  lock.unlock().unwrap();

  let progress_bar = ProgressBar::new(0);
  progress_bar.set_style(
    ProgressStyle::default_bar().template("Reading package information: {bar:40} {msg}"),
  );
  let resolved_items = &*source::CACHE;

  let progress_bar = ProgressBar::new(0);
  progress_bar.set_style(
    ProgressStyle::default_bar().template("Checking dpkg status       : {bar:40} {msg}"),
  );
  let upgradable_items = match dpkg::check_upgradable(&resolved_items, Some(&progress_bar)) {
    Ok(_upgradable_items) => _upgradable_items,
    Err(msg) => {
      println!("\n{}", msg);
      return;
    }
  };
  if upgradable_items.len() != 0 {
    println!(
      "{} packages are upgradable.",
      upgradable_items.len().to_string().red().bold()
    );
  } else {
    println!("{}", "All packages are up to date.".green().bold());
  }
}

pub fn fetch_indexes_thread(
  sources: &Vec<slist::Source>,
) -> Result<(Vec<u64>, Vec<SourcePackage>), String> {
  let mut handles = vec![];
  let mut package_items = vec![];
  let mut fetched_sizes = vec![];

  let (tx, rx) = mpsc::channel();
  let progress_bars = MultiProgress::new();
  let progress_style = ProgressStyle::default_bar()
    .template("Get: [{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
    .progress_chars("#>-");

  for ix in 0..sources.len() {
    let source = sources[ix].clone();
    let tx = tx.clone();
    let progress_bar = progress_bars.add(ProgressBar::new(9999999999));
    progress_bar.set_style(progress_style.clone());

    let handle = thread::spawn(move || {
      let raw_index = match fetcher::fetch_index(&source, Some(progress_bar)) {
        Ok(_raw_index) => _raw_index,
        Err(msg) => {
          println!("{}", msg);
          return;
        }
      };
      match cache::write_cache_raw(&raw_index, &source) {
        Ok(()) => {}
        Err(msg) => {
          println!("{}", msg);
          return;
        }
      }
      let fetched_size = raw_index.len() as u64;
      match source::SourcePackage::from_raw(&raw_index, &source.to_filename()) {
        Ok(mut _items) => {
          tx.send(Ok((fetched_size, _items))).unwrap();
        }
        Err(msg) => {
          tx.send(Err(msg)).unwrap();
        }
      }
    });
    handles.push(handle);
  }

  progress_bars.join().unwrap();
  for handle in handles {
    match rx.recv().unwrap() {
      Ok((fetched_size, mut item)) => {
        package_items.append(&mut item);
        fetched_sizes.push(fetched_size);
      }
      Err(msg) => {
        println!("{}", msg);
        return Err(msg);
      }
    }
    handle.join().unwrap();
  }

  Ok((fetched_sizes, package_items))
}
