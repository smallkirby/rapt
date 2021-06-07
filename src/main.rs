use simple_logger::SimpleLogger;

mod cli;
pub mod dpkg;
pub mod fetcher;
pub mod slist;
pub mod source;
pub mod update;

#[derive(Debug, PartialEq, Default)]
pub struct Opts {
  pub command: Command,
}

#[derive(Debug, PartialEq)]
pub enum Command {
  UPDATE,
  UNKNOWN,
}

impl Default for Command {
  fn default() -> Self {
    Self::UNKNOWN
  }
}

fn main() {
  let mut opts = Opts::default();

  println!("== rapt ==");
  SimpleLogger::new()
    .with_level(log::LevelFilter::Error)
    .init()
    .unwrap();

  parse_opts(&mut opts);

  match opts.command {
    Command::UPDATE => {
      update::do_update();
    }
    Command::UNKNOWN => {
      println!("Unknown subcommand");
    }
  }
}

pub fn parse_opts(opts: &mut Opts) {
  let matches = cli::build_cli().get_matches();

  if let Some(ref matches) = matches.subcommand_matches("update") {
    log::trace!("subcommand: update");
    opts.command = Command::UPDATE;
  } else {
    log::trace!("not implemented subcommand");
    opts.command = Command::UNKNOWN;
  }
}
