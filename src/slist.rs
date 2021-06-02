use std::path;
use std::fs;
use crate::helper;

#[derive(Debug, Default)]
pub struct pkgSourceList {

}

impl pkgSourceList {
  pub fn ReadMainList(&self) -> bool {
    unimplemented!();
  }

  pub fn ReadAppend(&self, file: &str) {
    if helper::flExtension(file) == "sources" {
      self.ParseFileDeb822(file);
    } else {
      self.ParseFileOldStyle(file);
    }
  }

  pub fn ParseFileDeb822(&self, file: &str) {
    log::trace!("parsing deb822 style sourcelist...");
    let sfile = fs::File::open(file);
    unimplemented!(); 
  }

  pub fn ParseFileOldStyle(&self, file: &str) {
    log::trace!("parsing old style sourcelist...");
    unimplemented!(); 
  }
}