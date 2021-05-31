use clap::{App, Arg};

#[derive(Debug, PartialEq)]
pub enum APT_CMD {
  APT,
  APT_GET,
}

pub struct CommandLine {}

pub struct AptDispatchWithHelp {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
  pub help: String,
}

pub struct Dispatch {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
}

impl AptDispatchWithHelp {
  pub fn new(
    com: impl Into<String>,
    handler: fn(&CommandLine) -> bool,
    help: impl Into<String>,
  ) -> AptDispatchWithHelp {
    AptDispatchWithHelp {
      com: com.into(),
      handler: handler,
      help: help.into(),
    }
  }
}

impl Dispatch {
  pub fn new(com: impl Into<String>, handler: fn(&CommandLine) -> bool) -> Dispatch {
    Dispatch {
      com: com.into(),
      handler: handler,
    }
  }
}

impl std::fmt::Debug for Dispatch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Dispatch")
      .field("com", &self.com)
      .field("handler", &"(unable to show)")
      .finish()
  }
}

impl CommandLine {}

pub fn GetCommands() -> Vec<AptDispatchWithHelp> {
  vec![
    AptDispatchWithHelp::new("list", DoList, "<help not imp>"),
    AptDispatchWithHelp::new("search", DoList, "<help not imp>"),
    AptDispatchWithHelp::new("help", DoList, "<help not imp>"),
  ]
}

pub fn GetCommand(cmds: &Vec<Dispatch>, cmdline: Vec<String>) -> Option<&Dispatch> {
  for s in cmdline {
    if s == "--" {
      return None;
    }
    for cmd in cmds {
      if cmd.com == s {
        return Some(cmd);
      }
    }
  }
  None
}

pub fn ParseCommandLine(cmdl: &mut CommandLine, binary: APT_CMD) -> Vec<Dispatch> {
  unimplemented!();
}

pub fn DoList(handler: &CommandLine) -> bool {
  unimplemented!();
}

pub fn DoSearch(handler: &CommandLine) -> bool {
  unimplemented!();
}

fn main() {
  println!("Toy Apt");

  let cmds_with_help = GetCommands();
  let cmds = cmds_with_help
    .into_iter()
    .map(|c| Dispatch::new(c.com, c.handler))
    .collect::<Vec<_>>();
  match GetCommand(&cmds, std::env::args().collect()) {
    Some(called_cmd) => {
      println!("{:?}", called_cmd);
    }
    None => {
      unimplemented!();
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_get_command_list() {
    let cmds = super::GetCommands();
    assert_eq!(cmds[0].com, "list");
  }

  #[test]
  fn parse_cmd_list() {
    let cmds_with_help = super::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      super::GetCommand(&cmds, vec![String::from("apt"), String::from("list")]).unwrap();
    assert_eq!(called_cmd.com, "list");
  }

  #[test]
  fn parse_cmd_help() {
    let cmds_with_help = super::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      super::GetCommand(&cmds, vec![String::from("apt"), String::from("help")]).unwrap();
    assert_eq!(called_cmd.com, "help");
  }
}
