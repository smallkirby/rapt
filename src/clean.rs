use colored::*;
use glob;

pub fn do_clean() {
  let mut sum_debs = 0;
  match glob::glob("archive/*") {
    Ok(paths) => {
      for entry in paths {
        match entry {
          Ok(path) => {
            sum_debs += 1;
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            match std::fs::remove_file(path) {
              Ok(()) => continue,
              Err(_) => {
                println!("Failed to delete an archive: {}", filename);
                return;
              }
            }
          }
          Err(msg) => {
            println!("failed to open ache archive file: {}", msg);
            return;
          }
        };
      }
    }
    Err(_) => {
      println!("invalid glob pattern.");
      return;
    }
  };

  println!("Cleared {} packages.", sum_debs.to_string().yellow().bold());
}
