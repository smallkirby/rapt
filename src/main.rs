use simple_logger::SimpleLogger;

fn main() {
  println!("== rapt ==");
  SimpleLogger::new()
    .with_level(log::LevelFilter::Trace)
    .init()
    .unwrap();
}