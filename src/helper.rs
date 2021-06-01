pub fn getBinName() -> String {
  let bpath = std::path::PathBuf::from(&std::env::args().collect::<Vec<_>>()[0]);
  String::from(bpath.file_name().unwrap().to_str().unwrap())
}
