use std::{cmp::Ordering, collections::HashMap};
use version_compare::{CompOp, Version, VersionCompare};

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
  pub chksum_md5: String,
  pub essential: bool,
  pub suggests: Vec<String>,
  pub filename: String,
  pub description: String,
}

impl SourcePackage {
  pub fn from_row(file: &str) -> Result<Vec<Self>, String> {
    let mut items = vec![];
    let mut item = SourcePackage::default();
    let lines = file.split("\n").collect::<Vec<_>>();
    let mut cont_description = false;
    let mut tmp_description = String::new();

    for (ix, line) in lines.iter().enumerate() {
      if cont_description {
        if ix < lines.len() - 1
          && lines
            .iter()
            .nth(ix + 1)
            .unwrap()
            .chars()
            .into_iter()
            .nth(0)
            .unwrap()
            == ' '
        {
          cont_description = true;
          tmp_description.push_str(&format!("\n{}", &line[1..]));
        } else {
          cont_description = false;
          tmp_description.push_str(&format!("\n{}", &line[1..]));
          item.description = tmp_description;
          tmp_description = String::new();
        }
        continue;
      }

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
        "Suggests" => {
          let _sug = parts.map(|s| String::from(*s)).collect::<Vec<_>>().join("");
          let sug = _sug.split(",").map(|s| s.trim()).collect::<Vec<_>>();
          for s in sug {
            item.suggests.push(s.to_string());
          }
        }
        "Breaks" => {
          //log::debug!("ignoring Breaks.");
        }
        "Filename" => {
          item.filename = parts
            .nth(0)
            .ok_or(format!("invalid 'Filename' format: {}", line))?
            .to_string();
        }
        "MD5sum" => {
          item.chksum_md5 = parts
            .nth(0)
            .ok_or(format!("invalid 'MD5sum' format: {}", line))?
            .to_string();
        }
        "Description" => {
          if ix < lines.len() - 1
            && lines
              .iter()
              .nth(ix + 1)
              .unwrap()
              .chars()
              .into_iter()
              .nth(0)
              .unwrap()
              == ' '
          {
            cont_description = true;
            tmp_description.push_str(
              &parts
                .map(|s| String::from(*s))
                .collect::<Vec<_>>()
                .join(" "),
            );
          } else {
            cont_description = false;
            tmp_description.push_str(
              &parts
                .map(|s| String::from(*s))
                .collect::<Vec<_>>()
                .join(" "),
            );
            item.description = tmp_description;
            tmp_description = String::new();
          }
        }
        _ => {
          //log::debug!(
          //  "{}: ignoring unknown package field: {}",
          //  item.package,
          //  title
          //);
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
  REQUIRED,
  IMPORTANT,
  STANDARD,
  OPTIONAL,
  EXTRA,
  UNKNOWN,
}

impl PartialOrd for Priority {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    use Ordering::*;
    use Priority::*;
    match (self, other) {
      (REQUIRED, REQUIRED) => Some(Equal),
      (REQUIRED, IMPORTANT) => Some(Greater),
      (REQUIRED, STANDARD) => Some(Greater),
      (REQUIRED, OPTIONAL) => Some(Greater),
      (REQUIRED, EXTRA) => Some(Greater),
      (REQUIRED, UNKNOWN) => Some(Greater),
      (IMPORTANT, REQUIRED) => Some(Less),
      (IMPORTANT, IMPORTANT) => Some(Equal),
      (IMPORTANT, STANDARD) => Some(Greater),
      (IMPORTANT, OPTIONAL) => Some(Greater),
      (IMPORTANT, EXTRA) => Some(Greater),
      (IMPORTANT, UNKNOWN) => Some(Greater),
      (STANDARD, REQUIRED) => Some(Less),
      (STANDARD, IMPORTANT) => Some(Less),
      (STANDARD, STANDARD) => Some(Equal),
      (STANDARD, OPTIONAL) => Some(Greater),
      (STANDARD, EXTRA) => Some(Greater),
      (STANDARD, UNKNOWN) => Some(Greater),
      (OPTIONAL, REQUIRED) => Some(Less),
      (OPTIONAL, IMPORTANT) => Some(Less),
      (OPTIONAL, STANDARD) => Some(Less),
      (OPTIONAL, OPTIONAL) => Some(Equal),
      (OPTIONAL, EXTRA) => Some(Greater),
      (OPTIONAL, UNKNOWN) => Some(Greater),
      (EXTRA, REQUIRED) => Some(Less),
      (EXTRA, IMPORTANT) => Some(Less),
      (EXTRA, STANDARD) => Some(Less),
      (EXTRA, OPTIONAL) => Some(Less),
      (EXTRA, EXTRA) => Some(Equal),
      (EXTRA, UNKNOWN) => Some(Greater),
      (UNKNOWN, REQUIRED) => Some(Less),
      (UNKNOWN, IMPORTANT) => Some(Less),
      (UNKNOWN, STANDARD) => Some(Less),
      (UNKNOWN, OPTIONAL) => Some(Less),
      (UNKNOWN, EXTRA) => Some(Less),
      (UNKNOWN, UNKNOWN) => Some(Equal),
    }
  }
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

pub fn resolve_duplication(sources: &Vec<SourcePackage>) -> Result<Vec<SourcePackage>, String> {
  let mut resolved: Vec<SourcePackage> = vec![];
  for source in sources {
    let dups_num = resolved
      .iter()
      .filter(|&item| item.package == source.package)
      .collect::<Vec<_>>()
      .len();
    if dups_num == 0 {
      resolved.push(source.clone());
    } else if dups_num != 1 {
      return Err("something went wrong while resolving duplication.".to_string());
    } else {
      let dup = resolved
        .iter()
        .find(|item| item.package == source.package)
        .unwrap();
      // check version and priority only
      if source.priority > dup.priority {
        let ix = resolved
          .iter()
          .position(|x| x.package == dup.package)
          .unwrap();
        resolved.push(source.clone());
        resolved.remove(ix);
      } else if source.priority < dup.priority {
        continue;
      } else {
        let version0 = Version::from(&dup.version);
        let version1 = Version::from(&source.version);
        match (version0, version1) {
          (None, _) => continue,
          (_, None) => {
            let ix = resolved
              .iter()
              .position(|x| x.package == dup.package)
              .unwrap();
            resolved.push(source.clone());
            resolved.remove(ix);
          }
          (Some(v0), Some(v1)) => {
            if v0 < v1 {
              let ix = resolved
                .iter()
                .position(|x| x.package == dup.package)
                .unwrap();
              resolved.push(source.clone());
              resolved.remove(ix);
            }
          }
        }
      }
    }
  }

  Ok(resolved)
}

#[cfg(test)]
pub mod test {
  #[test]
  fn test_package_source_from_row() {
    let sample = std::fs::read_to_string("test/sample-index").unwrap();
    let psources = super::SourcePackage::from_row(&sample).unwrap();
    let dpkg = &psources[0];
    assert_eq!(psources.len(), 3);
    assert_eq!(dpkg.package, "dpkg");
    assert_eq!(dpkg.pre_depends["libzstd1"].as_ref().unwrap(), "1.3.2");
    assert_eq!(dpkg.arch[0], super::Arch::AMD64);
    assert_eq!(dpkg.version, "1.19.7ubuntu3");
    assert_eq!(dpkg.essential, true);
    assert_eq!(dpkg.section, super::Section::ADMIN);
    assert_eq!(
      dpkg.maintainer,
      "Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>"
    );
    assert_eq!(
      dpkg.original_maintainer,
      "Dpkg Developers <debian-dpkg@lists.debian.org>"
    );
    assert_eq!(dpkg.suggests.contains(&"apt".to_string()), true);
    assert_eq!(dpkg.suggests.contains(&"debsig-verify".to_string()), true);
    assert_eq!(
      dpkg.filename,
      "pool/main/d/dpkg/dpkg_1.19.7ubuntu3_amd64.deb"
    );
    assert_eq!(dpkg.chksum_md5, "f595c79475d3c2ac808eaac389071c35");
    assert_eq!(
      dpkg.description,
      "Debian package management system\nwaiwai second sentence.\nuouo fish life."
    );
  }

  #[test]
  fn test_package_resolve_duplication() {
    let sample = std::fs::read_to_string("test/sample-duplicated-index").unwrap();
    let psources = super::SourcePackage::from_row(&sample).unwrap();
    assert_eq!(psources.len(), 3);
    let resolved = super::resolve_duplication(&psources).unwrap();
    assert_eq!(resolved.len(), 1);
    let dpkg = resolved.iter().nth(0).unwrap();
    assert_eq!(dpkg.package, "dpkg");
    assert_eq!(dpkg.priority, super::Priority::REQUIRED);
    assert_eq!(dpkg.version, "1.20.7ubuntu3");
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
