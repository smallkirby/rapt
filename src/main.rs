use simple_logger::SimpleLogger;

pub mod fetcher;
pub mod slist;

fn main() {
  println!("== rapt ==");
  SimpleLogger::new()
    .with_level(log::LevelFilter::Warn)
    .init()
    .unwrap();

  let source =
    &slist::parseSourceLine("deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted")
      .unwrap()[0];
  println!("{:?}", fetcher::fetchIndex(&source));
}
