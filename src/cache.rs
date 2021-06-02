use crate::slist;

#[derive(Debug, Default)]
pub struct CacheFile {
  pub sourcelist: slist::pkgSourceList,
}

impl CacheFile {
  pub fn GetSourceList() -> bool {

    unimplemented!();
  }

  pub fn BuildSourceList() {

  }
}