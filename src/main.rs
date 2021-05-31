use std::rc::Rc;
use std::cell::{Cell, RefCell};

pub mod wai;

#[derive(Debug, PartialEq)]
pub enum APT_CMD {
  APT,
  APT_GET,
}

type LinkNode = Option<Rc<RefCell<Item>>>;

#[derive(PartialEq)]
// represent list in same hierarchy
pub struct Item {
  parent: LinkNode,
  child: LinkNode,
  next: LinkNode,
  value: String,
  tag: String,
}

impl std::fmt::Debug for Item {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let parent_name = if let Some(p) = self.parent.as_ref() {
      String::from(&p.borrow().tag)
    } else {
      String::from("None")
    };
    let child_name = if let Some(p) = &self.child {
      String::from(&p.borrow().tag)
    } else {
      String::from("None")
    };
    write!(f, "ITEM {{tag: {:?}, value: {:?}, parent: {:?}, child: {:?}}}", self.tag, self.value, parent_name, child_name);
    Ok(())
  }
}

#[derive(Debug, PartialEq, Default)]
pub struct Configuration {
  root: LinkNode,
}

impl Configuration {
  pub fn new() -> Configuration { 
    Configuration {
        root: Some(Rc::new( RefCell::new(Item {
          parent: None,
          child: None,
          next: None,
          value: String::from(""),
          tag: String::from(""),
        }))),
    }
  }

  //pub fn set(&mut self, name: &str, value: &str) {
  //  if let Some(item) = self.lookup(name, true) {
  //    //item.value = value; 
  //  }
  //}

  pub fn push_child(&self, parent: Rc<RefCell<Item>>, val: &str, tag: &str) -> LinkNode {
    let child = Rc::new(RefCell::new(Item {
      parent: Some(parent.clone()),
      child: None,
      next: parent.borrow().child.clone(),
      value: String::from(val),
      tag: String::from(tag),
    }));
    parent.borrow_mut().child = Some(child.clone());
    Some(child)
  }

  // find the direct child with @tag
  pub fn lookup_child(&self, parent: &mut Rc<RefCell<Item>>, tag: &str, create: bool) -> LinkNode {
    let mut found = false;
    let mut cur_item = &parent.borrow_mut().child.clone();
    loop {
      match cur_item {
        Some(item) => {
          if item.borrow().tag == tag {
            found = true;
            return cur_item.clone();
          } else {
            cur_item = &item.clone().borrow_mut().next;
            //cur_item = &item.clone().borrow_mut().next;
            continue;
          }
        },
        None => break,
      }
    };

    if !create {
      None
    } else {
      let newitem = Rc::new(RefCell::new(Item{
        parent: None,
        child: None,
        next: None,
        value: String::from(""),
        tag: String::from(tag),
      }));
      match self.push_child(parent.clone(), "", tag) {
        Some(new) => Some(new),
        None => None,
      }
    }
  }

  // recursive lookup of Configuration.root
 // pub fn lookup(&self, name: &str, create: bool) -> Option<Box<Item>> {
 //   if name.len() == 0 { // terminator
 //     return Some(self.root.child);
 //   };
 //   for tag in name.split("::").collect::<Vec<_>>() {

 //   }
 // }
}


pub struct CommandLine {}

impl CommandLine {
  pub fn new() -> Self {
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
  pub fn new(name: impl Into<String>, short: impl Into<String>, long: impl Into<String>, flags: u64) -> Args {
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

pub fn BinaryCommandSpecificConfiguration(binary: &String, cmd: &String, config: &mut Configuration) {
  let binpath = std::path::PathBuf::from(&binary);
  match binpath.to_str().unwrap() {
    "apt" => {

    },
    _ => {
      unimplemented!();
    },
  }
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
      args.push(Args::new("APT::Cmd::ManualInstalled", "", "manual-installed", 0));
      args.push(Args::new("APT::Cmd::List-Include-Summary", "v", "verbose", 0));
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
  use std::rc::Rc;
  use std::cell::{Cell, RefCell};

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
    assert_eq!(called_cmd.com, "help");
  }

  #[test]
  fn test_push_child() {
    let n0 = Rc::new(RefCell::new(super::Item{
      parent: None,
      child: None,
      next: None,
      value: String::from("n0"),
      tag: String::from("N0"),
    }));
    let n1 = super::push_child(n0, "n1", "N1").unwrap();
    println!("{:?}", n1);
  }
}
