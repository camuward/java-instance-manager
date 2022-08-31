use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
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

    env_logger::builder()
        .parse_filters(
            &std::env::var_os("LOG_LEVEL")
                .unwrap_or_else(|| "off".into())
                .to_string_lossy(),
        )
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}",
                record.level().to_string().to_lowercase(),
                record.args()
            )
        })
        .init();

    // get our working directory
    let base = std::env::var_os("JIM_DIR") // first try env
        .map(|s| Path::new(&s).to_owned()) // convert osstring to path
        .or_else(|| Some(path_to_subdir(dirs::data_dir()?, "jim"))) // .local/share, %APPDATA%, etc.
        .map(dunce::canonicalize) // get full path without unc prefix
        .context("Failed to find data directory (hint: set `JIM_DIR` env var to override)")??;

    log::info!(
        "starting {} {} at {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        base.to_string_lossy()
    );

    // create our working directory if it doesn't already exist
    if !base.try_exists()? {
        log::info!("data directory doesn't exist! initializing...");
        log::debug!("creating default directory");
        fs::create_dir_all(&base)?;
        log::trace!("finished creating data directory");
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
            } else {
                log::info!("current instance symlink does not exist");
                std::process::exit(1);
            }
        }
        Command::Add { paths } => {
            for path in paths {
                let path = dunce::canonicalize(path)?;
                log::debug!(
                    "searching for input directory at {}...",
                    path.to_string_lossy()
                );
                anyhow::ensure!(path.try_exists()?, "Input path does not exist");

                let name = path
                    .file_name()
                    .context("Failed to get instance filename")?;

                let instance: PathBuf = path_to_subdir(&base, name);
                anyhow::ensure!(!instance.try_exists()?, "Instance is already installed");

                dircpy::CopyBuilder::new(&path, &instance)
                    .run()
                    .with_context(|| {
                        format!(
                            "Failed to copy instance from {} to {}",
                            path.to_string_lossy(),
                            instance.to_string_lossy()
                        )
                    })?;
            }
        }
        Command::Set { instance: name } => {
            log::debug!("searching for {}", name.to_string_lossy());
            let instance = path_to_subdir(&base, &name);
            anyhow::ensure!(
                instance.try_exists()?,
                "Instance {} does not exist",
                instance.to_string_lossy()
            );
            log::debug!("found {}", instance.to_string_lossy());

            log::info!("setting current instance to {}", name.to_string_lossy());

            let link = path_to_subdir(base, "current");
            if link.try_exists()? {
                log::debug!("symlink exists, removing...");
                symlink::remove_symlink_dir(&link).with_context(|| {
                    format!(
                        "Failed to remove `current` symlink ({})",
                        link.to_string_lossy()
                    )
                })?;
                log::trace!("symlink removed");
            }
            log::debug!("creating symlink...");
            symlink::symlink_dir(instance, &link).with_context(|| {
                format!(
                    "Failed to create `current` symlink ({})",
                    link.to_string_lossy()
                )
            })?;
            log::trace!("symlink created");
        }
    }

    Ok(())
}
