use crate::source::SourcePackage;
use version_compare::{CompOp, Version, VersionCompare};

pub fn read_dpkg_state() -> Result<Vec<SourcePackage>, String> {
  let raw_packages = match std::fs::read_to_string("/var/lib/dpkg/status") {
    Ok(_raw_packages) => _raw_packages,
    Err(_msg) => return Err(_msg.to_string()),
  };

  SourcePackage::from_row(&raw_packages)
}

pub fn check_upgradable(index_items: &Vec<SourcePackage>) -> Result<Vec<SourcePackage>, String> {
  let mut upgradable_items = vec![];

  let dpkg_items = match read_dpkg_state() {
    Ok(_dpkg_items) => _dpkg_items,
    Err(msg) => return Err(msg),
  };
  log::info!("dpkg has {} installed packages.", dpkg_items.len());

  for ditem in dpkg_items {
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

    if Version::from(&iitem.version) > Version::from(&ditem.version) {
      upgradable_items.push(ditem);
    }
  }

  Ok(upgradable_items)
}
