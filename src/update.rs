use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{mpsc, Mutex};
use std::thread;

use crate::cache;
use crate::dpkg;
use crate::fetcher;
use crate::slist;
use crate::source;
use crate::source::SourcePackage;

pub fn do_update() {
  log::trace!("do_update()");

  let mut package_items = vec![];

  // read sources.list
  let sources = match slist::parseSourceFile("sources.list") {
    Ok(_items) => _items,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  let start_time = std::time::SystemTime::now();
  // fetch index files and get package items.
  println!("Fetching indexes... ");
  let mut fetched_amount = 0;
  match fetche_indexes_thread(&sources) {
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

  print!("Reading package lists... ");
  let resolved_items = match source::resolve_duplication(&package_items) {
    Ok(_resolved_items) => _resolved_items,
    Err(msg) => {
      println!("\n{}", msg);
      return;
    }
  };
  println!("DONE");

  print!("Reading state information... ");
  let upgradable_items = match dpkg::check_upgradable(&resolved_items) {
    Ok(_upgradable_items) => _upgradable_items,
    Err(msg) => {
      println!("\n{}", msg);
      return;
    }
  };
  println!("DONE");
  if upgradable_items.len() != 0 {
    println!(
      "{} packages are upgradable.",
      upgradable_items.len().to_string().red().bold()
    );
  } else {
    println!("{}", "All packages are up to date.".green().bold());
  }
}

pub fn fetche_indexes_thread(
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
      //println!("Get:{} {}", ix, source.info());
      let raw_index = match fetcher::fetchIndex(&source, &progress_bar) {
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
      match source::SourcePackage::from_row(&raw_index) {
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
