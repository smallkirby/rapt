// @ret: 1 iif vs1 > vs2, -1 iif vs1 < vs2, 0 iif vs1 == vs2
// here, a>b means that a is newer than b.
pub fn comp_version(vs1: &str, vs2: &str) -> i32 {
  let reg = regex::Regex::new(r"^([0-9a-zA-Z\.\+\-\~]+)\-([0-9a-zA-Z\+\.~]+)$").unwrap();
  let (epoc1, upstream1, revision1) = split_version(vs1, &reg);
  let (epoc2, upstream2, revision2) = split_version(vs2, &reg);

  // compare epoc
  match (epoc1, epoc2) {
    (None, None) => {}
    (None, Some(_epoc2_val)) => return -1,
    (Some(_epoc1_val), None) => return 1,
    (Some(epoc1_val), Some(epoc2_val)) => {
      if epoc1_val > epoc2_val {
        return 1;
      } else if epoc1_val < epoc2_val {
        return -1;
      }
    }
  }

  // compare upstream_version
  let cmp_upstream_res = comp_upstream_version(&upstream1, &upstream2);
  if cmp_upstream_res > 0 {
    return 1;
  } else if cmp_upstream_res < 0 {
    return -1;
  }

  // here, both must have revision, compare it.
  let rev1 = match revision1 {
    Some(_rev1) => _rev1,
    None => "".to_string(),
  };
  let rev2 = match revision2 {
    Some(_rev2) => _rev2,
    None => "".to_string(),
  };
  comp_upstream_version(&rev1, &rev2)
}

pub fn comp_upstream_version(vs1: &str, vs2: &str) -> i32 {
  let _parts1 = split_in_upstream(vs1);
  let _parts2 = split_in_upstream(vs2);
  let mut parts1 = _parts1.iter();
  let mut parts2 = _parts2.iter();

  loop {
    // loop for each digit,nondigit pairs
    let (a1, b1) = match parts1.next() {
      Some((_a, _b)) => (_a.clone(), _b.clone()),
      None => ("".to_string(), "".to_string()),
    };
    let (a2, b2) = match parts2.next() {
      Some((_a, _b)) => (_a.clone(), _b.clone()),
      None => ("".to_string(), "".to_string()),
    };
    if a1.len() == 0 && b1.len() == 0 && a2.len() == 0 && b2.len() == 0 {
      break;
    }

    // compare digit part
    let a1n: u32 = match a1.len() {
      0 => 0,
      _ => a1.parse().unwrap(),
    };
    let a2n: u32 = match a2.len() {
      0 => 0,
      _ => a2.parse().unwrap(),
    };
    if a1n > a2n {
      return 1;
    } else if a1n < a2n {
      return -1;
    }

    // compare non-digit part
    let mut b1iter = b1.chars();
    let mut b2iter = b2.chars();
    loop {
      let b1c = b1iter.next();
      let b2c = b2iter.next();
      if b1c.is_none() && b2c.is_none() {
        break;
      }

      let cmp_res = comp_version_char(b1c, b2c);
      if cmp_res > 0 {
        return 1;
      } else if cmp_res < 0 {
        return -1;
      }
    }
  }

  0
}

// @ret: 1 if c1 is more new
#[allow(non_snake_case)]
pub fn comp_version_char(c1: Option<char>, c2: Option<char>) -> i32 {
  let MAGIC = -0x50;
  let HYPHEN = -0x100;
  let EMPTY = -0xFF;
  let n1 = match c1 {
    Some(c) => {
      if c == '~' {
        HYPHEN
      } else {
        if c.is_alphabetic() {
          c as i32
        } else {
          (c as i32) - MAGIC
        }
      }
    }
    None => EMPTY,
  };
  let n2 = match c2 {
    Some(c) => {
      if c == '~' {
        HYPHEN
      } else {
        if c.is_alphabetic() {
          c as i32
        } else {
          (c as i32) - MAGIC
        }
      }
    }
    None => EMPTY,
  };

  if n1 > n2 {
    1
  } else if n1 < n2 {
    -1
  } else {
    0
  }
}

pub fn split_in_upstream(vs: &str) -> Vec<(String, String)> {
  let mut ret = vec![];
  let mut now_digit = String::new();
  let mut now_nondigit = String::new();
  let mut in_nondigit = false;
  for c in vs.chars() {
    if c.is_digit(10) {
      if in_nondigit {
        ret.push((now_digit.clone(), now_nondigit.clone()));
        now_digit = String::new();
        now_nondigit = String::new();
        now_digit.push(c);
        in_nondigit = false;
      } else {
        now_digit.push(c);
      }
    } else {
      now_nondigit.push(c);
      in_nondigit = true;
    }
  }
  if now_digit.len() != 0 || now_nondigit.len() != 0 {
    ret.push((now_digit, now_nondigit));
  }

  ret
}

pub fn split_version(vs: &str, reg: &regex::Regex) -> (Option<u32>, String, Option<String>) {
  let v;
  let epoc_ix = vs.find(':');
  let epoc = match epoc_ix {
    Some(epoc_ix) => {
      v = String::from(&vs[epoc_ix + 1..]);
      Some(vs[..epoc_ix].parse().unwrap())
    }
    None => {
      v = String::from(vs);
      None
    }
  };

  let mut hyphen_occ = vec![];
  for (ix, c) in v.chars().enumerate() {
    if c == '-' {
      hyphen_occ.push(ix);
    }
  }

  match hyphen_occ.len() {
    0 => {
      return (epoc, v, None);
    }
    1 => {
      let parts = v.split("-").collect::<Vec<_>>();
      return (epoc, String::from(parts[0]), Some(String::from(parts[1])));
    }
    _ => {
      // XXX compiling same regex is really wastefull
      let parts = reg.captures_iter(&v).collect::<Vec<_>>()[0]
        .iter()
        .collect::<Vec<_>>();
      return (
        epoc,
        String::from(parts[1].unwrap().as_str()),
        Some(String::from(parts[2].unwrap().as_str())),
      );
    }
  }
}

#[cfg(test)]
pub mod test {
  #[test]
  fn test_versions() {
    use super::comp_version;
    assert_eq!(comp_version("30:ubuntu03.a3", "30:ubuntu03.a3"), 0);
    assert_eq!(comp_version("32:ubuntu03.a3", "30:ubuntu03.a3"), 1);
    assert_eq!(comp_version("ubuntu03.a3", "ubuntu03.a3~beta"), 1);
    assert_eq!(
      comp_version("20200323-1build1~ubuntu20.04.1", "20200323-1"),
      1
    );
    assert_eq!(comp_version("1:3.36.5-0ubuntu2", "1:3.36.5-0ubuntu1"), 1);
  }

  #[test]
  fn test_split_version() {
    use super::split_version;
    let reg = regex::Regex::new(r"^([0-9a-zA-Z\.\+\-\~]+)\-([0-9a-zA-Z\+\.~]+)$").unwrap();
    let p1 = split_version("12:1.27.1ubuntu2", &reg);
    let p2 = split_version("1.27.1ubuntu2", &reg);
    let p3 = split_version("12:1.27.1ubuntu2-0.30", &reg);
    let p4 = split_version("12:1.27.1ubuntu2-0.30ubuntu~beta", &reg);
    let p5 = split_version("12:1.27.1ubuntu2+4.3-0.30ubuntu~beta", &reg);
    let p6 = split_version("12:1.27.1ubuntu-2+4.3-0.30ubuntu~beta", &reg);
    assert_eq!(p1, (Some(12), "1.27.1ubuntu2".to_string(), None));
    assert_eq!(p2, (None, "1.27.1ubuntu2".to_string(), None));
    assert_eq!(
      p3,
      (
        Some(12),
        "1.27.1ubuntu2".to_string(),
        Some("0.30".to_string())
      )
    );
    assert_eq!(
      p4,
      (
        Some(12),
        "1.27.1ubuntu2".to_string(),
        Some("0.30ubuntu~beta".to_string())
      )
    );
    assert_eq!(
      p5,
      (
        Some(12),
        "1.27.1ubuntu2+4.3".to_string(),
        Some("0.30ubuntu~beta".to_string())
      )
    );
    assert_eq!(
      p6,
      (
        Some(12),
        "1.27.1ubuntu-2+4.3".to_string(),
        Some("0.30ubuntu~beta".to_string())
      )
    );
  }
}
