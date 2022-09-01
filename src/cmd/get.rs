use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::Context;

pub fn get(base: impl AsRef<Path>) -> anyhow::Result<()> {
    let link = crate::path_to_subdir(base, "current");

    let real_path = link
        .try_exists()?
        .then(|| fs::read_link(link))
        .transpose()?
        .context("No instance set")?;

    // print instance name
    real_path
        .file_name()
        .context("Failed to read instance folder name")
        .map(OsStr::to_string_lossy)
        .map(|dir_name| println!("{dir_name}"))
}
