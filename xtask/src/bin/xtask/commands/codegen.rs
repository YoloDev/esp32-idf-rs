use std::{env, fs, path::Path};

use clap::Clap;
use glob::glob;
use log::info;
use xtask::codegen;

use super::XTask;

#[derive(Clap)]
pub(crate) struct Codegen {}

impl XTask for Codegen {
  fn run(&self) -> anyhow::Result<()> {
    let idf_path = env::var("IDF_PATH")?;
    let idf_path: &Path = idf_path.as_ref();
    let mut includes = glob(idf_path.join("components/**/include").to_str().unwrap())?
      .collect::<Result<Vec<_>, _>>()?;
    includes.push(fs::canonicalize("script/include")?);
    let includes = includes.iter().map(AsRef::as_ref).collect::<Vec<_>>();

    for path in glob("sys/*/Bindings.toml")? {
      let path = path?;
      info!("Generating bindings from {}", path.display());
      codegen::gen_bindings(&path, &includes)?;
    }

    Ok(())
  }
}
