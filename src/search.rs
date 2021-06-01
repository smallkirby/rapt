use crate::cmdline;

pub fn FullTextSearch(cmdl: &cmdline::CommandLine) -> bool {
  unimplemented!();
}

pub fn DoSearch(cmdl: &cmdline::CommandLine) -> bool {
  log::trace!("search command starts");
  FullTextSearch(cmdl)
}
