use clap::{App, Arg, ArgGroup, SubCommand};

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
          Arg::with_name("upgradable")
            .help("show only upgradable packages.")
            .long("upgradable"),
        )
        .group(ArgGroup::with_name("list-option").args(&["installed", "upgradable"]))
        .arg(Arg::with_name("package").help("target package glob term to search for")),
      SubCommand::with_name("search")
        .about("search package database for specific term in package name or its description, and list packages.")
        .arg(Arg::with_name("package").help("target package regex to search for").required(true))
        .arg(Arg::with_name("full-text").help("show full description of packages.").short("f").long("full-text")),
      SubCommand::with_name("show")
        .about("search package database for specific glob term in package name and show detailed information.")
        .arg(Arg::with_name("package").help("target package glob to search for").required(true)),
      SubCommand::with_name("install")
        .about("install package")
        .arg(Arg::with_name("package").help("package name or .deb file to install").required(true)),
    ])
}
