use crate::configuration;
use crate::helper;
use crate::list;
use crate::search;

#[derive(Debug, PartialEq)]
pub enum APT_CMD {
  APT,
  APT_GET,
}

pub struct Dispatch {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
}

pub struct AptDispatchWithHelp {
  pub com: String,
  pub handler: fn(&CommandLine) -> bool,
  pub help: String,
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

#[derive(Debug, PartialEq, Default)]
pub struct Args {
  name: String,
  short: String,
  long: String,
  flags: u64,
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
          Err(msg) => {
            println!("Err: {}", msg);
            std::process::exit(100);
          }
        }

        return cmds;
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

  pub fn DispatchCommandLine(&mut self, cmds: Vec<Dispatch>) {
    log::trace!("dispatching list: {:?}", cmds);
    for cmd in cmds {
      if self.FileList[0] == cmd.com {
        log::trace!("matched command: {:?}", cmd);
        if !(cmd.handler)(self) {
          unimplemented!();
        }
      }
    }
    unimplemented!();
  }
}

pub fn GetCommands() -> Vec<AptDispatchWithHelp> {
  vec![
    AptDispatchWithHelp::new("list", list::DoList, "<help not imp>"),
    AptDispatchWithHelp::new("search", search::DoSearch, "<help not imp>"),
    AptDispatchWithHelp::new("help", list::DoList, "<help not imp>"),
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
