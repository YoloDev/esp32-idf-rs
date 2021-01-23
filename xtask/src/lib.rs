#![feature(command_access)]
#![feature(str_split_once)]

pub mod codegen;
mod fs;

pub use fs::{find_workspace, get_idf_env};
