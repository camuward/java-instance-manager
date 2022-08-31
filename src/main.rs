use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List the installed Java instances.
    List,

    /// Add an instance to the manager.
    Add {
        #[clap(value_parser)]
        path: PathBuf,
    },

    /// Get the current Java instance.
    Get,

    /// Set the current Java instance.
    Set {
        #[clap(value_parser)]
        instance: OsString,
    },
}

fn path_to_subdir(base: impl AsRef<Path>, ext: impl AsRef<OsStr>) -> PathBuf {
    base.as_ref()
        .components()
        .chain([Component::Normal(ext.as_ref())])
        .collect()
}

fn path_to_current_instance(base: impl AsRef<Path>) -> anyhow::Result<Option<PathBuf>> {
    let link = path_to_subdir(base, "current");

    Ok(link
        .try_exists()?
        .then(|| fs::read_link(link))
        .transpose()?)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // get our working directory
    let base = std::env::var_os("JIM_DIR") // first try env
        .map(|s| Path::new(&s).to_owned())
        .or_else(|| Some(path_to_subdir(dirs::data_dir()?, "jim"))) // .local/share, %APPDATA%, etc.
        .context("Failed to find data directory (hint: set `JIM_DIR` env var to override)")?;

    // create our working directory if it doesn't already exist
    if !dbg!(&base).try_exists()? {
        fs::create_dir_all(&base)?;
    }

    match cli.command {
        Command::List => {
            for instance in fs::read_dir(&base)?.flatten() {
                let meta = instance.metadata()?;
                if meta.is_dir() && !meta.is_symlink() {
                    println!("{}", instance.file_name().to_string_lossy());
                }
            }
        }
        Command::Get => {
            if let Some(path) = path_to_current_instance(&base)? {
                let name = path
                    .file_name()
                    .context("Failed to get instance filename")?
                    .to_string_lossy();

                println!("{name}");
            }
        }
        Command::Add { path } => {
            anyhow::ensure!(path.try_exists()?, "Input path does not exist");

            let name = path
                .file_name()
                .context("Failed to get instance filename")?;

            let instance: PathBuf = path_to_subdir(&base, name);
            anyhow::ensure!(!instance.try_exists()?, "Instance is already installed");

            fs_extra::dir::copy(&path, &instance, &Default::default())?;
        }
        Command::Set { instance } => {
            let instance = path_to_subdir(&base, instance);
            anyhow::ensure!(
                instance.try_exists()?,
                "Instance {} does not exist",
                instance.to_string_lossy()
            );

            let link = path_to_subdir(base, "current");
            if link.try_exists()? {
                symlink::remove_symlink_dir(&link).with_context(|| {
                    format!(
                        "Failed to remove `current` symlink ({})",
                        link.to_string_lossy()
                    )
                })?;
            }
            symlink::symlink_dir(instance, &link).with_context(|| {
                format!(
                    "Failed to create `current` symlink ({})",
                    link.to_string_lossy()
                )
            })?;
        }
    }

    Ok(())
}
