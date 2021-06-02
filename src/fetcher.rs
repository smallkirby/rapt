use crate::slist;
use bytes::{BytesMut, BufMut};
use curl::easy::Easy;
use std::io::prelude::*;
use flate2::read::GzDecoder;

pub fn fetchIndex(source: &slist::Source) -> String {
  let indexuri = source.toIndexUri();
  let mut easy = Easy::new();
  let mut buf;
  easy.url(&indexuri).unwrap();
  easy.write_function(move |data| {
    buf = data;
    Ok(data.len())
  }).unwrap();
  easy.perform().unwrap();
  //println!("{:?}", buf);
  println!("{:?}", buf);
  //let mut decorder = GzDecoder::new(buf);
  //let mut buf = String::new();
  //decorder.read_to_string(&mut buf).unwrap();

  //buf
  "".to_string()
}

#[cfg(test)]
pub mod test {
  #[test]
  fn test_fetchIndex() {
    use crate::slist;
    let source = &slist::parseSourceLine("deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted").unwrap()[0];
    println!("{:?}", super::fetchIndex(&source));
  }
}