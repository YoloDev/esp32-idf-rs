use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize, Debug)]
struct Config {
  component: String,
  headers: Vec<String>,
  functions: Vec<String>,
}

pub fn gen_bindings(conf_file: &Path, idf_path: &Path, includes: &[&Path]) -> Result<()> {
  let dir = conf_file.parent().unwrap();
  let src_dir = dir.join("src");
  let content = fs::read_to_string(conf_file)?;
  let config: Config = toml::from_str(&content)?;
  debug!("bindgen: {:#?}", config);

  let mut builder = bindgen::Builder::default();
  for header in config.headers {
    let header = idf_path
      .join("components")
      .join(&config.component)
      .join("include")
      .join(header);
    builder = builder.header(header.to_str().unwrap());
    //writeln!(file, "#include \"{}\"", header)?;
  }

  // let builder = builder.header(header_file.path().to_str().unwrap());
  let mut builder = builder
    .raw_line("#![allow(non_camel_case_types, non_upper_case_globals)]")
    .rustfmt_bindings(true)
    .rustfmt_configuration_file(Some(fs::canonicalize("rustfmt.toml").unwrap()))
    .size_t_is_usize(true)
    .use_core()
    .ctypes_prefix("::cty");

  for fun in config.functions {
    builder = builder.whitelist_function(fun);
  }

  let out = builder
    .clang_arg("-D__GLIBC_USE(x)=0")
    .clang_arg("-DSSIZE_MAX")
    .clang_args(includes.iter().map(|i| format!("-I{}", i.display())))
    .generate_block(true)
    .generate()
    .unwrap();

  let out_file = src_dir.join("bindings.rs");
  info!("Writing to file: {}", out_file.display());
  out.write_to_file(&out_file)?;
  Ok(())
}
