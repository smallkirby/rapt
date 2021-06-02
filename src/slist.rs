#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SourceType {
  DEB,
  DEBSRC,
}

impl Default for SourceType {
  fn default() -> Self {
    SourceType::DEB
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Protocol {
  HTTP,
}

impl Default for Protocol {
  fn default() -> Self {
    Protocol::HTTP
  }
}

#[derive(Debug, PartialEq, Default)]
pub struct Source {
  stype: SourceType,
  protocol: Protocol,
  uri: String,
  dists: String,
  component: String,
}

impl Source {
  pub fn toIndexUri(&self) -> String {
    let mut iuri = String::new();
    match self.protocol {
      Protocol::HTTP => iuri.push_str("http"),
      _ => unimplemented!(),
    };
    iuri.push_str("://");
    iuri.push_str(&self.uri);
    if self.uri.chars().nth(self.uri.len() - 1).unwrap() != '/' {
      iuri.push_str("/");
    };
    iuri.push_str(&format!("dists/{}/", self.dists));
    iuri.push_str(&format!("{}/", self.component));
    iuri.push_str("binary-amd64/");
    iuri.push_str("Packages.gz");

    iuri
  }
}

pub fn parseSourceLine(line: &str) -> Result<Vec<Source>, String> {
  let parts = line.split(" ").collect::<Vec<_>>();
  if parts.len() < 4 {
    return Err(String::from("Malformed source line."));
  }
  let stype = match parts[0] {
    "deb" => SourceType::DEB,
    "deb-src" => SourceType::DEBSRC,
    _ => return Err(format!("Unknown source type: {}", parts[0])),
  };
  let _uri = parts[1].split("://").collect::<Vec<_>>();
  if _uri.len() != 2 {
    return Err(format!("Malformed source line: invalid uri: {}", parts[1]));
  }
  let protocol = match _uri[0] {
    "http" => Protocol::HTTP,
    _ => {
      return Err(format!(
        "Malformed source line: invalid protocol: {}",
        _uri[0]
      ))
    }
  };
  let uri = _uri[1];
  let dists = parts[2];
  let components = parts[3..].iter().collect::<Vec<_>>();

  Ok(
    components
      .iter()
      .map(|component| Source {
        stype: stype,
        protocol: protocol,
        uri: uri.to_string(),
        dists: dists.to_string(),
        component: component.to_string(),
      })
      .collect::<Vec<_>>(),
  )
}

#[cfg(test)]
pub mod tests {
  #[test]
  pub fn test_parseSourceLine() {
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted";
    let s1 = super::Source {
      stype: super::SourceType::DEB,
      protocol: super::Protocol::HTTP,
      uri: "jp.archive.ubuntu.com/ubuntu/".to_string(),
      dists: "focal".to_string(),
      component: "main".to_string(),
    };
    let s2 = super::Source {
      stype: super::SourceType::DEB,
      protocol: super::Protocol::HTTP,
      uri: "jp.archive.ubuntu.com/ubuntu/".to_string(),
      dists: "focal".to_string(),
      component: "restricted".to_string(),
    };
    let sources = super::parseSourceLine(line).unwrap();
    assert_eq!(sources[0], s1);
    assert_eq!(sources[1], s2);
  }

  #[test]
  pub fn test_toIndexUri() {
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal main";
    let source = &super::parseSourceLine(line).unwrap()[0];
    let uri = source.toIndexUri();
    assert_eq!(
      uri,
      "http://jp.archive.ubuntu.com/ubuntu/dists/focal/main/binary-amd64/Packages.gz"
    );
  }
}
