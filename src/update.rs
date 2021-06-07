use colored::*;

use crate::cache;
use crate::dpkg;
use crate::fetcher;
use crate::slist;
use crate::source;

pub fn do_update() {
  log::trace!("do_update()");

  let mut fetched_amount: u64 = 0;
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
  for (ix, source) in sources.iter().enumerate() {
    println!("Get:{} {}", ix, source.info());
    let raw_index = match fetcher::fetchIndex(&source) {
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
    fetched_amount += raw_index.len() as u64;
    println!(
      "{}:{} {} [{} B]",
      "Hit".blue(),
      ix,
      source.info(),
      raw_index.len()
    );
    match source::SourcePackage::from_row(&raw_index) {
      Ok(mut _items) => {
        log::info!("fetched {} packages.", _items.len());
        package_items.append(&mut _items);
      }
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    }
  }
  let total_time = start_time.elapsed().unwrap().as_secs();
  let fetched_amount_kb: u64 = (fetched_amount / 1024).into();
  println!(
    "Fetched {} kB in {}s ({} kB/s)",
    fetched_amount_kb,
    total_time,
    fetched_amount_kb / total_time
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
