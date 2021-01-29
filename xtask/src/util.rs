use std::{
  collections::{HashMap, HashSet, VecDeque},
  env,
  ffi::OsStr,
  fs,
  io::{self, BufRead},
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Result;
use cargo_metadata::{CargoOpt, MetadataCommand};
use duct::cmd;
use io::BufReader;
use log::{debug, trace};
use petgraph::{algo::toposort, Graph};
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

pub fn set_var(name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
  let name = name.as_ref();
  let value = value.as_ref();
  trace!("{}={}", name.to_str().unwrap(), value.to_str().unwrap());
  env::set_var(name, value)
}

pub fn get_idf_env(idf_path: Option<&Path>) -> Result<()> {
  let idf_path = match idf_path {
    None => fs::canonicalize("esp-idf")?,
    Some(p) => p.to_owned(),
  };
  set_var("IDF_PATH", &idf_path);

  let reader = cmd!(
    "python",
    idf_path.join("tools/idf_tools.py"),
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

  cmd!(
    "python",
    idf_path.join("tools/check_python_dependencies.py")
  )
  .before_spawn(log_cmd)
  .reader()?;

  Ok(())
}

pub struct CrateInfo {
  name: String,
  manifest_path: PathBuf,
}

impl CrateInfo {
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn manifest(&self) -> &Path {
    &self.manifest_path
  }

  // pub fn dir(&self) -> &Path {
  //   self.manifest_path.parent().unwrap()
  // }
}

pub fn get_crates() -> Result<impl Iterator<Item = CrateInfo>> {
  let metadata = MetadataCommand::new()
    .no_deps()
    .features(CargoOpt::AllFeatures)
    .exec()?;

  let workspace_member_ids = metadata
    .workspace_members
    .into_iter()
    .collect::<HashSet<_>>();

  let packages = metadata
    .packages
    .into_iter()
    .filter(move |p| workspace_member_ids.contains(&p.id))
    .filter(|p| p.name != "xtask")
    .collect::<Vec<_>>();

  let name_lookup = packages
    .iter()
    .map(|p| (p.name.clone(), p.id.clone()))
    .collect::<HashMap<_, _>>();

  let deps = packages
    .iter()
    .map(|p| {
      (
        p.id.clone(),
        p.dependencies
          .iter()
          .filter_map(|d| name_lookup.get(&d.name).cloned())
          .collect::<Vec<_>>(),
      )
    })
    .collect::<HashMap<_, _>>();

  let mut graph = Graph::new();
  let pkg_indices = packages
    .into_iter()
    .map(|p| (p.id.clone(), graph.add_node(p)))
    .collect::<HashMap<_, _>>();

  for (pkg_id, deps) in deps {
    let pkg_index = pkg_indices[&pkg_id];
    for dep in deps {
      let dep_index = pkg_indices[&dep];
      graph.add_edge(pkg_index, dep_index, ());
    }
  }

  graph.reverse();
  let sorting = toposort(&graph, None).unwrap();
  let packages = sorting.into_iter().map(move |idx| {
    let p = &graph[idx];
    CrateInfo {
      name: p.name.clone(),
      manifest_path: p.manifest_path.clone(),
    }
  });

  Ok(packages)
}
