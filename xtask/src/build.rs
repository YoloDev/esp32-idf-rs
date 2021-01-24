use std::path::Path;

use anyhow::Result;
use duct::cmd;
use log::info;

use crate::util;

pub fn build_libs(rust_xtansa_dir: &Path) -> Result<()> {
  util::set_var(
    "RUSTC",
    rust_xtansa_dir.join("build/x86_64-unknown-linux-gnu/stage2/bin/rustc"),
  );
  util::set_var(
    "RUSTDOC",
    rust_xtansa_dir.join("build/x86_64-unknown-linux-gnu/stage2/bin/rustdoc"),
  );
  util::set_var("XARGO_RUST_SRC", rust_xtansa_dir.join("library"));

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
