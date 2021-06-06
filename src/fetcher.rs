use crate::slist;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub fn fetchIndex(source: &slist::Source) -> Result<String, String> {
  let indexuri = source.toIndexUri();
  let mut res = reqwest::blocking::get(indexuri).expect("unknown error while fetching index.");
  if !res.status().is_success() {
    return Err(format!(
      "error while fetching index: error code={}",
      res.status().as_str()
    ));
  }
  let mut buf: Vec<u8> = vec![];
  res
    .copy_to(&mut buf)
    .expect("error while copying result into buffer.");

  let mut d = GzDecoder::new(&buf[..]);
  let mut s = String::new();
  d.read_to_string(&mut s).unwrap();

  Ok(s)
}

#[cfg(test)]
pub mod test {
  fn test_fetchIndex() {
    use crate::slist;
    let source =
      &slist::parseSourceLine("deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted")
        .unwrap()[0];
    println!("{}", super::fetchIndex(&source).unwrap());
  }
}
