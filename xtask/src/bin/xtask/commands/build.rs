use std::path::PathBuf;

use clap::{Clap, ValueHint};
use xtask::build;

use super::XTask;

#[derive(Clap)]
pub(crate) struct Build {
  #[clap(
    name = "rust-xtensa-path",
    long = "rust-xtensa-path",
    env = "RUST_XTENSA_PATH",
    parse(from_os_str),
    value_hint = ValueHint::FilePath)]
  rust_xtensa_path: Option<PathBuf>,

  #[clap(
    name = "rust-xtensa-bin",
    long = "rust-xtensa-bin",
    env = "RUST_XTENSA_BIN",
    parse(from_os_str),
    value_hint = ValueHint::FilePath,
    required_unless_present = "rust-xtensa-path")]
  rust_xtensa_bin: Option<PathBuf>,

  #[clap(
    name = "rust-xtensa-lib",
    long = "rust-xtensa-lib",
    env = "RUST_XTENSA_LIB",
    parse(from_os_str),
    value_hint = ValueHint::FilePath,
    required_unless_present = "rust-xtensa-path")]
  rust_xtensa_lib: Option<PathBuf>,
}

impl XTask for Build {
  fn run(&self) -> anyhow::Result<()> {
    let bin_dir = self.rust_xtensa_bin.clone().unwrap_or_else(|| {
      self
        .rust_xtensa_path
        .as_ref()
        .unwrap()
        .join("build/x86_64-unknown-linux-gnu/stage2/bin")
    });
    let lib_dir = self
      .rust_xtensa_lib
      .clone()
      .unwrap_or_else(|| self.rust_xtensa_path.as_ref().unwrap().join("library"));
    build::build_libs(&bin_dir, &lib_dir)
  }
}
