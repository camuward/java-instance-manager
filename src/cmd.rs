use std::fs;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::Subcommand;

pub use list::*;
pub use add::*;
pub use get::*;
pub use set::*;

pub mod list;
pub mod add;
pub mod get;
pub mod set;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List the installed Java instances.
    List,

    /// Add an instance to the manager.
    Add {
        #[clap(value_parser)]
        paths: Vec<PathBuf>,
    },

    /// Get the current Java instance.
    Get,

    /// Set the current Java instance.
    Set {
        #[clap(value_parser)]
        instance: OsString,
    },
}

pub fn path_to_current_instance(base: impl AsRef<Path>) -> anyhow::Result<Option<PathBuf>> {
    let link = crate::path_to_subdir(base, "current");

    Ok(link
        .try_exists()?
        .then(|| fs::read_link(link))
        .transpose()?)
}
