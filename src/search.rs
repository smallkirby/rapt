use crate::cmdline;

pub fn FullTextSearch(cmdl: &cmdline::CommandLine) -> bool {
  // XXX read/generate caches

  if cmdl.FileList.len() < 1 {
    println!("You must give at least one search pattern.");
  }

  unimplemented!();
}

pub fn DoSearch(cmdl: &cmdline::CommandLine) -> bool {
  log::trace!("search command starts");
  FullTextSearch(cmdl)
}
