use simple_logger::SimpleLogger;

pub mod fetcher;
pub mod slist;

fn main() {
  println!("== rapt ==");
  SimpleLogger::new()
    .with_level(log::LevelFilter::Trace)
    .init()
    .unwrap();
}
