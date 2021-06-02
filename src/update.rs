use crate::cmdline;
use crate::update;
use crate::slist;
use crate::cache;

pub fn DoUpdate(cmdl: &cmdline::CommandLine) -> bool {
  log::trace!("DoUpdate() {:?}", cmdl.FileList);
  if cmdl.FileList.len() != 1 {
    println!("The update command takes no arguments.");
    return false;
  }

  let cache = cache::CacheFile {..Default::default()};

  let sourcelist = slist::pkgSourceList {};

  unimplemented!();
}