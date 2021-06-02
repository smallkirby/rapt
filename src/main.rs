pub mod cmdline;
pub mod configuration;
pub mod helper;
pub mod list;
pub mod search;
pub mod cache;
pub mod prog;
pub mod update;
pub mod slist;

use cmdline::{AptDispatchWithHelp, Args, CommandLine, Dispatch, APT_CMD};
use simple_logger::SimpleLogger;

pub struct APT {
  config: configuration::Configuration,
  cmdl: CommandLine,
}

impl APT {
  pub fn run(&mut self) {
    let cmds = self.cmdl.parse(APT_CMD::APT, &mut self.config);
    self.cmdl.DispatchCommandLine(cmds);
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
  use crate::cmdline;

  #[test]
  fn test_get_command_list() {
    let cmds = cmdline::GetCommands();
    assert_eq!(cmds[0].com, "list");
  }

  #[test]
  fn parse_cmd_list() {
    use crate::cmdline;
    let cmds_with_help = cmdline::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      cmdline::GetCommand(&cmds, vec![String::from("apt"), String::from("list")]).unwrap();
    assert_eq!(called_cmd, "list");
  }

  #[test]
  #[should_panic]
  fn parse_cmd_not_exist() {
    use crate::cmdline;
    let cmds_with_help = cmdline::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      cmdline::GetCommand(&cmds, vec![String::from("apt"), String::from("nirugiri")]).unwrap();
  }

  #[test]
  fn parse_cmd_help() {
    use crate::cmdline;
    let cmds_with_help = cmdline::GetCommands();
    let cmds = cmds_with_help
      .into_iter()
      .map(|c| super::Dispatch::new(c.com, c.handler))
      .collect::<Vec<_>>();
    let called_cmd =
      cmdline::GetCommand(&cmds, vec![String::from("apt"), String::from("help")]).unwrap();
    assert_eq!(called_cmd, "help");
  }
}
