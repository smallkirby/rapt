use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SourcePackage {
  pub package: String,
  pub binary: Vec<String>,
  pub arch: Vec<Arch>,
  pub version: String,
  pub priority: Priority,
  pub section: Section,
  pub maintainer: String,
  pub original_maintainer: String,
  pub uploaders: Vec<String>,
  pub standard_version: String,
  pub pre_depends: HashMap<String, Option<String>>,
  pub testsuite: String,
  pub homepage: String,
  pub directory: String,
  pub chksum_sha1: String,
  pub essential: bool,
  pub suggests: Vec<String>,
}

impl SourcePackage {
  pub fn from_row(file: &str) -> Result<Vec<Self>, String> {
    let mut items = vec![];
    let mut item = SourcePackage::default();
    let lines = file.split("\n").collect::<Vec<_>>();

    for line in lines {
      if line.len() == 0 {
        match item.verify() {
          Ok(()) => {
            items.push(item.clone());
            item = SourcePackage::default();
            continue;
          }
          Err(msg) => return Err(msg),
        }
      }
      let _parts = line.split(": ").collect::<Vec<_>>();
      let mut parts = _parts.iter();
      let title = parts.nth(0).unwrap();
      match *title {
        "Package" => {
          item.package = parts
            .nth(0)
            .ok_or(format!("invalid 'Package' format: {}", line))?
            .to_string();
        }
        "Architecture" => {
          let arch = parts
            .nth(0)
            .ok_or(format!("invalid 'Architecture' format: {}", line))?;
          item.arch.push(match *arch {
            "amd64" => Arch::AMD64,
            "all" => Arch::ALL,
            "any" => Arch::ANY,
            _ => Arch::UNKNOWN,
          });
        }
        "Version" => {
          item.version = parts
            .nth(0)
            .ok_or(format!("invalid 'Version' format: {}", line))?
            .to_string();
        }
        "Priority" => {
          item.priority = match *parts
            .nth(0)
            .ok_or(format!("invalid 'Priority' format: {}", line))?
          {
            "extra" => Priority::EXTRA,
            "optional" => Priority::OPTIONAL,
            "important" => Priority::IMPORTANT,
            "required" => Priority::REQUIRED,
            "standard" => Priority::STANDARD,
            _ => Priority::UNKNOWN,
          };
        }
        "Essential" => {
          item.essential = match *parts
            .nth(0)
            .ok_or(format!("invalid 'Essential' format: {}", line))?
          {
            "yes" => true,
            _ => false,
          };
        }
        "Section" => {
          item.section = match *parts
            .nth(0)
            .ok_or(format!("invalid 'Section' format: {}", line))?
          {
            "admin" => Section::ADMIN,
            _ => Section::UNKNOWN, // XXX
          };
        }
        "Maintainer" => {
          item.maintainer = parts
            .map(|s| String::from(*s))
            .collect::<Vec<_>>()
            .join(" ");
        }
        "Original-Maintainer" => {
          item.original_maintainer = parts
            .map(|s| String::from(*s))
            .collect::<Vec<_>>()
            .join(" ");
        }
        "Pre-Depends" => {
          let _depends = parts.map(|s| String::from(*s)).collect::<Vec<_>>().join("");
          let depends = _depends.split(",").collect::<Vec<_>>();
          for dep in depends {
            match parse_depends(dep) {
              Ok((pkg, version)) => item.pre_depends.insert(pkg, version),
              Err(msg) => return Err(msg),
            };
          }
        }
        _ => {
          log::debug!("ignoring unknown package field: {}", title);
        }
      }
    }

    Ok(items)
  }

  pub fn verify(&self) -> Result<(), String> {
    Ok(())
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Arch {
  ALL,
  ANY,
  AMD64,
  UNKNOWN,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Priority {
  EXTRA,
  IMPORTANT,
  OPTIONAL,
  REQUIRED,
  STANDARD,
  UNKNOWN,
}

impl Default for Priority {
  fn default() -> Self {
    Self::UNKNOWN
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Section {
  ADMIN,
  COMM,
  DATABASE,
  DEBUG,
  DEVEL,
  DOC,
  EDITORS,
  FONTS,
  GAMES,
  GNOME,
  GRAPHICS,
  HTTPD,
  INTERPRETERS,
  INTROSPECTION,
  JAVA,
  KERNEL,
  LIBDEVEL,
  LIBS,
  LISP,
  LOCALIZATION,
  MAIL,
  MATH,
  METAPACKAGES,
  MISC,
  NET,
  OLDLIBS,
  OTHEROSFS,
  PERL,
  PHP,
  PYTHON,
  RUBY,
  SCIENCE,
  SHELLS,
  SOUND,
  TEXT,
  TRANSLATIONS,
  UTILS,
  VCS,
  VIDEO,
  WEB,
  X11,
  ZOPE,
  UNKNOWN,
}

impl Default for Section {
  fn default() -> Self {
    Self::UNKNOWN
  }
}

pub fn parse_depends(_dep: &str) -> Result<(String, Option<String>), String> {
  let dep: String = _dep.chars().filter(|c| !c.is_whitespace()).collect();
  if dep.contains("(") {
    // XXX assumes version format is like: (>=2.32)
    let pkg = &dep[0..dep.find("(").unwrap()];
    let version = &dep[dep.find("(").unwrap() + 1 + 2..dep.len() - 1];
    Ok((pkg.to_string(), Some(version.to_string())))
  } else {
    Ok((dep, None))
  }
}

#[cfg(test)]
pub mod test {
  #[test]
  fn test_package_source_from_row() {
    let sample = std::fs::read_to_string("test/sample-index").unwrap();
    let psources = super::SourcePackage::from_row(&sample).unwrap();
    assert_eq!(psources.len(), 3);
    assert_eq!(psources[0].package, "dpkg");
    assert_eq!(
      psources[0].pre_depends["libzstd1"].as_ref().unwrap(),
      "1.3.2"
    );
  }

  #[test]
  fn test_parse_depends() {
    let dep1 = "libc6 (>= 2.15)";
    let dep2 = "libbz2-1.0";
    let pdep1 = super::parse_depends(dep1).unwrap();
    let pdep2 = super::parse_depends(dep2).unwrap();
    assert_eq!(pdep1.0, "libc6");
    assert_eq!(pdep1.1.unwrap(), "2.15");
    assert_eq!(pdep2.0, "libbz2-1.0");
    assert_eq!(pdep2.1, None);
  }
}
