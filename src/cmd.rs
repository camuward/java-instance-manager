use std::ffi::OsString;
use std::path::PathBuf;

use clap::Subcommand;

pub use list::*;
pub use add::*;
pub use get::*;
pub use set::*;

mod list;
mod add;
mod get;
mod set;

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
