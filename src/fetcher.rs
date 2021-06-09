use crate::slist;
use crate::source;
use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{header, Client, Url};
use std::io::prelude::*;

pub fn fetch_deb(package: &source::SourcePackage) -> Result<String, String> {
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

  Ok(debname)
}

pub fn fetchIndex(
  source: &slist::Source,
  _progress_bar: Option<ProgressBar>,
) -> Result<String, String> {
  let mut buf: Vec<u8> = vec![];
  let indexuri = source.toIndexUri();
  match _progress_bar {
    Some(progress_bar) => {
      let _indexuri = indexuri.clone();
      let task = async {
        let client = Client::new();
        let download_size = {
          let res = client.head(_indexuri.clone()).send().await.unwrap();
          if res.status().is_success() {
            res
              .headers()
              .get(header::CONTENT_LENGTH)
              .and_then(|ct_len| ct_len.to_str().ok())
              .and_then(|ct_len| ct_len.parse().ok())
              .unwrap_or(0)
          } else {
            return Err("unknown error while fetching index.".to_string());
          }
        };

        progress_bar.set_message(_indexuri.clone());
        progress_bar.set_length(download_size);
        progress_bar.set_position(0);

        // actual request
        let req = client.get(_indexuri.clone());
        let mut download = req.send().await.unwrap();
        while let Some(chunk) = download.chunk().await.unwrap() {
          progress_bar.inc(chunk.len() as u64);
          tokio::io::AsyncWriteExt::write(&mut buf, &chunk)
            .await
            .unwrap();
        }

        progress_bar.finish();
        Ok(())
      };

      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(task).unwrap();

      let mut d = GzDecoder::new(&buf[..]);
      let mut s = String::new();
      d.read_to_string(&mut s).unwrap();

      Ok(s)
    }
    None => {
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
  }
}

#[cfg(test)]
pub mod test {
  fn test_fetchIndex() {
    use crate::slist;
    let source =
      &slist::parseSourceLine("deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted")
        .unwrap()[0];
    println!("{}", super::fetchIndex(&source, None).unwrap());
  }

  fn test_fetch_deb() {
    let p = crate::source::SourcePackage {
      package: "vim".to_string(),
      filename: "pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb".to_string(),
      ..Default::default()
    };
    super::fetch_deb(&p);
  }
}
