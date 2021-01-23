use std::{
  collections::{HashMap, VecDeque},
  env,
  ffi::OsStr,
  fs,
  io::{self, BufRead},
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Result;
use duct::cmd;
use env::{join_paths, split_paths};
use io::BufReader;
use log::{debug, trace};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct LocateProject<'a> {
  root: &'a str,
}

#[allow(clippy::unnecessary_wraps)]
fn log_cmd(cmd: &mut Command) -> io::Result<()> {
  let mut builder: String = cmd.get_program().to_string_lossy().into();
  let args = cmd.get_args().map(|arg| arg.to_string_lossy());
  for arg in args {
    builder.reserve(arg.len() + 3);
    builder.push(' ');
    builder.push('"');
    builder += &arg;
    builder.push('"');
  }

  debug!("running: {}", builder);

  Ok(())
}

pub fn find_workspace() -> Result<PathBuf> {
  let json = cmd!("cargo", "locate-project", "--workspace")
    .before_spawn(log_cmd)
    .read()?;

  let parsed = serde_json::from_str::<LocateProject>(&json)?;
  let path: &Path = parsed.root.as_ref();
  Ok(path.parent().unwrap().to_owned())
}

macro_rules! join_p_str {
  ($p:expr $(,)?) => {
    AsRef::<Path>::as_ref(&$p).to_owned()
  };
  ($p1:expr, $p2:expr $(,)?) => {
    join_p_str!($p1.join($p2))
  };
  ($p1:expr, $p2:expr, $($pr:expr),+$(,)?) => {
    join_p_str!($p1.join($p2), $($pr,)+)
  };
}

fn set_var(name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
  let name = name.as_ref();
  let value = value.as_ref();
  trace!("{}={}", name.to_str().unwrap(), value.to_str().unwrap());
  env::set_var(name, value)
}

pub fn get_idf_env() -> Result<()> {
  let idf_path = fs::canonicalize("esp-idf")?;
  set_var("IDF_PATH", &idf_path);

  let reader = cmd!(
    "python",
    "esp-idf/tools/idf_tools.py",
    "export",
    "--format",
    "key-value"
  )
  .before_spawn(log_cmd)
  .reader()?;

  let mut path = {
    let path = env::var("PATH")?;
    env::split_paths(&path).collect::<VecDeque<_>>()
  };

  for line in BufReader::new(reader).lines() {
    let line = line?;
    let (name, val) = line.split_once('=').unwrap();
    if name == "PATH" {
      #[cfg(not(target_os = "windows"))]
      const STR: &str = ":$PATH";
      #[cfg(target_os = "windows")]
      const STR: &str = ";%PATH%";

      let new_path = val.trim_end_matches(STR);
      for part in env::split_paths(new_path)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
      {
        path.push_front(part)
      }
    } else {
      set_var(name, val);
    }
  }

  let components = idf_path.join("components");
  path.push_front(join_p_str!(components, "esptool_py", "esptool"));
  path.push_front(join_p_str!(components, "app_update"));
  path.push_front(join_p_str!(components, "espcoredump"));
  path.push_front(join_p_str!(components, "partition_table"));

  let path = env::join_paths(path)?;
  set_var("PATH", path);

  cmd!("python", "esp-idf/tools/check_python_dependencies.py")
    .before_spawn(log_cmd)
    .reader()?;

  Ok(())
}
