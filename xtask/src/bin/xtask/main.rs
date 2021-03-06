#![feature(command_access)]

use std::{
  env,
  path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Clap, ValueHint};
use commands::XTask;
use log::debug;
use options::{ColorChoice, Verbosity};
use tracing::metadata::LevelFilter;
use xtask::{find_workspace, get_idf_env};

mod commands;
mod logging;
mod options;

impl Verbosity {
  fn as_filter(self) -> Option<LevelFilter> {
    match self {
      Verbosity::Default => None,
      Verbosity::Off => Some(LevelFilter::OFF),
      Verbosity::Error => Some(LevelFilter::ERROR),
      Verbosity::Warn => Some(LevelFilter::WARN),
      Verbosity::Info => Some(LevelFilter::INFO),
      Verbosity::Debug => Some(LevelFilter::DEBUG),
      Verbosity::Trace => Some(LevelFilter::TRACE),
    }
  }
}

impl ColorChoice {
  fn as_termcolor(self) -> termcolor::ColorChoice {
    match self {
      ColorChoice::Default => termcolor::ColorChoice::Auto,
      ColorChoice::Force => termcolor::ColorChoice::Always,
      ColorChoice::ForceAnsi => termcolor::ColorChoice::AlwaysAnsi,
      ColorChoice::Disable => termcolor::ColorChoice::Never,
    }
  }
}

#[derive(Clap)]
struct Options {
  #[clap(flatten)]
  verbosity: Verbosity,

  #[clap(flatten)]
  color: ColorChoice,

  #[clap(subcommand)]
  command: commands::Command,

  #[clap(
    name = "idf-path",
    long = "idf-path",
    env = "IDF_PATH",
    parse(from_os_str),
    value_hint = ValueHint::FilePath)]
  idf_path: Option<PathBuf>,
}

fn main() -> Result<()> {
  let opts = Options::parse();
  let color = opts.color.as_termcolor();
  let filter = opts.verbosity.as_filter().unwrap_or(LevelFilter::INFO);

  logging::init(color, filter);

  let workspace = find_workspace()?;
  debug!("cwd = {}", workspace.display());
  env::set_current_dir(&workspace)?;

  get_idf_env(opts.idf_path.as_ref().map(AsRef::<Path>::as_ref))?;

  // info!("workspace: {}", workspace.display());
  opts.command.run()
}
