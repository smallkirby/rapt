use crate::slist;
use crate::source;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub fn fetch_deb(package: &source::SourcePackage) -> Result<(), String> {
  // create archive directory
  if !std::path::Path::new("archive").exists() {
    std::fs::create_dir("archive").unwrap();
  }

  /* XXX must check archive directory first for cache */

  let uri = match package.toPoolUri() {
    Ok(_uri) => _uri,
    Err(()) => {
      return Err(format!(
        "failed to open cache or package not found in caches: {}",
        package.package
      ))
    }
  };

  let res = reqwest::blocking::get(&uri).expect("unknown error while fetching package.");
  if !res.status().is_success() {
    return Err(format!(
      "error while fetching index: error code={}",
      res.status().as_str()
    ));
  }
  let _a = uri.rfind('/').unwrap();
  let debname = String::from(&uri[_a + 1..]);
  let mut output = std::fs::File::create(format!("archive/{}", debname)).unwrap();
  let content = res.bytes().unwrap();
  output.write_all(&content).unwrap();

  Ok(())
}

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

  #[test]
  fn test_fetch_deb() {
    let p = crate::source::SourcePackage {
      package: "vim".to_string(),
      filename: "pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb".to_string(),
      ..Default::default()
    };
    super::fetch_deb(&p);
  }
}
