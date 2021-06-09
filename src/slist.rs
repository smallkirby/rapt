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

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Source {
  stype: SourceType,
  protocol: Protocol,
  uri: String,
  dists: String,
  component: String,
}

impl Source {
  pub fn to_filename(&self) -> String {
    format!(
      "{}_dists_{}-{}",
      self.uri.replace("/", "_"),
      self.dists,
      self.component
    )
  }

  pub fn info(&self) -> String {
    let proto = match self.protocol {
      Protocol::HTTP => "http",
    };
    format!("{}://{} {} {}", proto, self.uri, self.dists, self.component)
  }

  pub fn to_index_uri(&self) -> String {
    let mut iuri = String::new();
    match self.protocol {
      Protocol::HTTP => iuri.push_str("http"),
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

pub fn parse_source_file(filename: &str) -> Result<Vec<Source>, String> {
  let mut sources = vec![];
  let source_lines = if let Ok(_s) = std::fs::read_to_string("sources.list") {
    _s
  } else {
    return Err(format!("Failed to open source list file: {}", filename));
  };
  for line in source_lines.split("\n").collect::<Vec<_>>() {
    if line.len() != 0 {
      match parse_source_line(line) {
        Ok(mut items) => sources.append(&mut items),
        Err(msg) => return Err(msg),
      }
    };
  }

  Ok(sources)
}

pub fn parse_source_line(line: &str) -> Result<Vec<Source>, String> {
  if line.len() == 0 {
    return Ok(vec![]);
  }
  if line.chars().nth(0).unwrap() == '#' {
    return Ok(vec![]);
  }
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
  pub fn test_parse_source_line() {
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
    let sources = super::parse_source_line(line).unwrap();
    assert_eq!(sources[0], s1);
    assert_eq!(sources[1], s2);
  }

  #[test]
  pub fn test_to_index_uri() {
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal main";
    let source = &super::parse_source_line(line).unwrap()[0];
    let uri = source.to_index_uri();
    assert_eq!(
      uri,
      "http://jp.archive.ubuntu.com/ubuntu/dists/focal/main/binary-amd64/Packages.gz"
    );
  }
}
