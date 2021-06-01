use simple_logger::SimpleLogger;

pub mod configuration;
pub mod helper;

#[derive(Debug, PartialEq)]
pub enum APT_CMD {
  APT,
  APT_GET,
}

#[derive(Debug, Default)]
pub struct CommandLine {
  ArgList: Vec<Args>,
  FileList: Vec<String>,
}

impl CommandLine {
  pub fn new() -> CommandLine {
    CommandLine {
      ..Default::default()
    }
  }

  pub fn parse(
    &mut self,
    binary: APT_CMD,
    config: &mut configuration::Configuration,
  ) -> Vec<Dispatch> {
    log::warn!("should call pkgInitConfig()"); // XXX
    config.BinarySpecificConfiguration(&std::env::args().collect::<Vec<_>>()[0]);

    let cmds_with_help = GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    match GetCommand(&cmds, std::env::args().collect()) {
      Some(called_cmd) => {
        log::trace!("cmd: {:?}", called_cmd);
        config.BinaryCommandSpecificConfiguration(
          &std::env::args().collect::<Vec<_>>()[0],
          &called_cmd,
        );

        // XXX needed???
        config.MoveSubTree(&format!("Binary::{}", helper::getBinName()), "");

        let args = GetCommandArgs(APT_CMD::APT, &called_cmd);
        log::trace!("args: {:?}", args);
        self.ArgList = args;

        match self.doParse(&std::env::args().collect::<Vec<_>>()[1..]) {
          Ok(()) => {}
          Err(msg) => panic!(msg), // XXX shouldn't panic
        }
      }
      None => {
        unimplemented!();
      }
    };

    vec![]
  }

  // main func of parse command line
  // @return: true iif success
  pub fn doParse(&mut self, cargs: &[String]) -> Result<(), String> {
    for opt in cargs {
      // not an option
      if opt.chars().nth(0).unwrap() != '-' {
        self.FileList.push(String::from(opt));
        continue;
      }

      if let Some(c) = opt.chars().nth(1) {
        if c == '-' && opt.len() == 2 {
          // two dashes mean end of option processing
          unimplemented!();
        } else if c == '-' {
          // long option
          log::trace!("long option processing");
          unimplemented!();
        } else {
          log::trace!("short option processing");
          for c in opt[1..].chars() {
            let matched_opt = self
              .ArgList
              .iter()
              .filter(|op| op.short == String::from(c))
              .collect::<Vec<_>>();
            if matched_opt.len() == 0 {
              return Err(format!("Command line option '{}' [from {}] is not understood in combination with the other options.", c, opt));
            } else {
              self.handleOpt();
            }
          }
          continue;
        }
      }
    }
    Ok(())
  }

  // helper function
  pub fn handleOpt(&mut self) {
    unimplemented!();
  }
}

pub struct AptDispatchWithHelp {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
  pub help: String,
}

pub struct Dispatch {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
}

#[derive(Debug, PartialEq, Default)]
pub struct Args {
  name: String,
  short: String,
  long: String,
  flags: u64,
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

impl Args {
  pub fn new(
    name: impl Into<String>,
    short: impl Into<String>,
    long: impl Into<String>,
    flags: u64,
  ) -> Args {
    Args {
      name: name.into(),
      short: short.into(),
      long: long.into(),
      flags: flags,
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

pub fn GetCommands() -> Vec<AptDispatchWithHelp> {
  vec![
    AptDispatchWithHelp::new("list", DoList, "<help not imp>"),
    AptDispatchWithHelp::new("search", DoList, "<help not imp>"),
    AptDispatchWithHelp::new("help", DoList, "<help not imp>"),
  ]
}

pub fn GetCommand(cmds: &Vec<Dispatch>, cmdline: Vec<String>) -> Option<String> {
  for s in cmdline {
    if s == "--" {
      return None;
    }
    for cmd in cmds {
      if cmd.com == s {
        return Some(String::from(&cmd.com));
      }
    }
  }
  None
}

pub fn GetCommandArgs(program: APT_CMD, cmd: &String) -> Vec<Args> {
  let mut args = vec![];

  if cmd != "help" {
    // for now, assume that argv[0]=="apt"("rapt")
    if cmd == "list" {
      args.push(Args::new("APT::Cmd::Installed", "i", "installed", 0));
      args.push(Args::new("APT::Cmd::Upgradable", "", "upgradable", 0));
      args.push(Args::new("APT::Cmd::Upgradable", "", "upgradeable", 0));
      args.push(Args::new("APT::Cmd::Upgradable", "u", "upgradable", 0));
      args.push(Args::new(
        "APT::Cmd::ManualInstalled",
        "",
        "manual-installed",
        0,
      ));
      args.push(Args::new(
        "APT::Cmd::List-Include-Summary",
        "v",
        "verbose",
        0,
      ));
      args.push(Args::new("APT::Cmd::AllVersions", "a", "all-versions", 0));
    }
  }

  AddDefaultArgs(&mut args);

  args
}

pub fn AddDefaultArgs(args: &mut Vec<Args>) {
  args.push(Args::new("help", "h", "help", 0));
  args.push(Args::new("version", "v", "version", 0));
}

pub fn DoList(handler: &CommandLine) -> bool {
  unimplemented!();
}

pub fn DoSearch(handler: &CommandLine) -> bool {
  unimplemented!();
}

pub struct APT {
  config: configuration::Configuration,
  cmdl: CommandLine,
}

impl APT {
  pub fn run(&mut self) {
    self.cmdl.parse(APT_CMD::APT, &mut self.config);
  }
}

fn main() {
  println!("Toy Apt");
  SimpleLogger::new()
    .with_level(log::LevelFilter::Trace)
    .init()
    .unwrap();

  let mut apt = APT {
    config: configuration::Configuration::new(),
    cmdl: CommandLine::new(),
  };
  apt.run();
}

#[cfg(test)]
mod tests {
  use crate::configuration;

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
    assert_eq!(called_cmd, "list");
  }

  #[test]
  #[should_panic]
  fn parse_cmd_not_exist() {
    let cmds_with_help = super::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      super::GetCommand(&cmds, vec![String::from("apt"), String::from("nirugiri")]).unwrap();
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
    assert_eq!(called_cmd, "help");
  }
}
