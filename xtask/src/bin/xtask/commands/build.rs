use std::path::PathBuf;

use clap::{Clap, ValueHint};
use xtask::build;

use super::XTask;

#[derive(Clap)]
pub(crate) struct Build {
  #[clap(long = "rust-xtensa-path", env = "RUST_XTENSA_PATH", parse(from_os_str), value_hint = ValueHint::FilePath)]
  rust_xtensa_path: PathBuf,
}

impl XTask for Build {
  fn run(&self) -> anyhow::Result<()> {
    build::build_libs(&self.rust_xtensa_path)
  }
}
