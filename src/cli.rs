use clap::{App, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
  App::new("rapt")
    .version(env!("CARGO_PKG_VERSION"))
    .author("(c) 2021 Nirugiri")
    .subcommands(vec![SubCommand::with_name("update").about("update index")])
}
