use colored::*;
use simple_logger::SimpleLogger;

pub mod cache;
pub mod clean;
mod cli;
pub mod dpkg;
pub mod fetcher;
pub mod install;
pub mod list;
pub mod lock;
pub mod search;
pub mod show;
pub mod slist;
pub mod source;
pub mod update;
pub mod version;

#[derive(Debug, PartialEq, Default)]
pub struct Opts {
  pub command: Command,
  pub installed: bool,
  pub upgradabe: bool,
  pub full_description: bool,
  pub package: String,
}

#[derive(Debug, PartialEq)]
pub enum Command {
  UPDATE,
  LIST,
  SEARCH,
  SHOW,
  INSTALL,
  CLEAN,
  UNKNOWN,
}

impl Default for Command {
  fn default() -> Self {
    Self::UNKNOWN
  }
}

fn main() {
  let mut opts = Opts::default();

  println!(" {}\n", "======== RAPT =======".blue().bold());
  SimpleLogger::new()
    .with_level(log::LevelFilter::Error)
    .init()
    .unwrap();

  parse_opts(&mut opts);

  match opts.command {
    Command::UPDATE => {
      update::do_update();
    }
    Command::LIST => {
      list::do_list(&opts.package, opts.installed, opts.upgradabe);
    }
    Command::SEARCH => {
      search::do_search(&opts.package, opts.full_description);
    }
    Command::SHOW => {
      show::do_show(&opts.package);
    }
    Command::INSTALL => {
      install::do_install(&opts.package);
    }
    Command::CLEAN => {
      clean::do_clean();
    }
    Command::UNKNOWN => {
      println!("Unknown subcommand");
    }
  }
}

pub fn parse_opts(opts: &mut Opts) {
  let matches = cli::build_cli().get_matches();

  if let Some(ref _matches) = matches.subcommand_matches("update") {
    log::trace!("subcommand: update");
    opts.command = Command::UPDATE;
  } else if let Some(ref matches) = matches.subcommand_matches("list") {
    log::trace!("subcommand: list");
    opts.command = Command::LIST;
    opts.package = match matches.value_of("package") {
      Some(_package) => _package.to_string(),
      None => "*".to_string(),
    };
    if matches.is_present("installed") {
      opts.installed = true;
    }
    if matches.is_present("upgradable") {
      opts.upgradabe = true;
    }
    log::trace!("option: installed: {:?}", opts.installed);
    log::trace!("package: {}", opts.package);
  } else if let Some(ref matches) = matches.subcommand_matches("search") {
    log::trace!("subcommand: search");
    opts.command = Command::SEARCH;
    opts.package = matches.value_of("package").unwrap().to_string();
    if matches.is_present("full-text") {
      opts.full_description = true;
    }
    log::trace!("package: {}", opts.package);
  } else if let Some(ref matches) = matches.subcommand_matches("show") {
    log::trace!("subcommand: show");
    opts.command = Command::SHOW;
    opts.package = matches.value_of("package").unwrap().to_string();
    log::trace!("package: {}", opts.package);
  } else if let Some(ref _matches) = matches.subcommand_matches("clean") {
    log::trace!("subcommand: clean");
    opts.command = Command::CLEAN;
  } else if let Some(ref matches) = matches.subcommand_matches("install") {
    log::trace!("subcommand: install");
    opts.command = Command::INSTALL;
    opts.package = matches.value_of("package").unwrap().to_string();
    log::trace!("package: {}", opts.package);
  } else {
    log::trace!("not implemented subcommand");
    opts.command = Command::UNKNOWN;
  }
}
