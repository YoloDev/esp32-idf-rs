use anyhow::Result;
use clap::Clap;

mod build;
mod codegen;

pub(crate) trait XTask: Clap {
  fn run(&self) -> Result<()>;
}

#[derive(Clap)]
pub(crate) enum Command {
  /// Run codegen
  Codegen(codegen::Codegen),

  /// Build libraries against esp32
  Build(build::Build),
}

impl XTask for Command {
  fn run(&self) -> Result<()> {
    match self {
      Command::Codegen(x) => x.run(),
      Command::Build(x) => x.run(),
    }
  }
}
