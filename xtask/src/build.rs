use std::path::Path;

use anyhow::Result;
use duct::cmd;
use log::info;

use crate::util;

pub fn build_libs(rust_bin: &Path, rust_lib: &Path) -> Result<()> {
  util::set_var("RUSTC", rust_bin.join("rustc"));
  util::set_var("RUSTDOC", rust_bin.join("rustdoc"));
  util::set_var("XARGO_RUST_SRC", rust_lib);

  for c in util::get_crates()? {
    info!("Building {}", c.name());
    cmd!(
      "cargo",
      "xbuild",
      "--manifest-path",
      c.manifest(),
      "--target",
      "xtensa-esp32-none-elf",
      "--release"
    )
    .run()?;
  }

  Ok(())
}
