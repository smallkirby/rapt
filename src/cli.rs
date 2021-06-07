use clap::{App, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
  App::new("rapt")
    .version(env!("CARGO_PKG_VERSION"))
    .author("(c) 2021 Nirugiri")
    .subcommands(vec![
      SubCommand::with_name("update").about("update index"),
      SubCommand::with_name("list")
        .about("query package database and list packages.")
        .arg(
          Arg::with_name("installed")
            .help("show only installed packages.")
            .long("installed"),
        )
        .arg(
          Arg::with_name("package")
            .help("target package to search for")
            .required(true),
        ),
    ])
}
