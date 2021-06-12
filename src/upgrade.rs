use crate::{cache, dpkg, install, source};
use indicatif::{ProgressBar, ProgressStyle};

pub fn do_upgrade() {
  let progress_bar = ProgressBar::new(0);
  progress_bar.set_style(
    ProgressStyle::default_bar().template("Checking dpkg status       : {bar:40} {msg}"),
  );

  let cached_items = &*source::CACHE;
  let upgradable_items_info = match dpkg::check_upgradable(&cached_items, Some(&progress_bar)) {
    Ok(_upgradable_items) => _upgradable_items,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };
  let upgradable_items = cache::search_cache_with_names(
    &upgradable_items_info
      .iter()
      .map(|i| i.package.clone())
      .collect::<Vec<_>>(),
  );

  match install::install_packages(upgradable_items.iter().map(|u| u).collect::<Vec<_>>()) {
    Ok(_) => {}
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  }
}
