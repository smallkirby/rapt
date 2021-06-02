pub fn getBinName() -> String {
  let bpath = std::path::PathBuf::from(&std::env::args().collect::<Vec<_>>()[0]);
  String::from(bpath.file_name().unwrap().to_str().unwrap())
}

pub fn str2bool(val: &str, default: bool) -> bool {
  match val {
    "no" | "false" | "without" | "off" | "disable" => false,
    "yes" | "true" | "with" | "on" | "enable" => true,
    _ => default,
  }
}

// return file extension
pub fn flExtension(file: &str) -> String {
  let fpath = std::path::PathBuf::from(file);
  let fparts = fpath.file_name().unwrap().to_str().unwrap().split(".").collect::<Vec<_>>();
  if fparts.len() == 1 {
    String::from("")
  } else {
    String::from(*fparts.iter().nth(fparts.len() - 1).unwrap())
  }
}