use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::cmd::Command;

mod cmd;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

pub(crate) fn extend_path<S: AsRef<OsStr>>(
    parent: impl AsRef<Path>,
    children: impl AsRef<[S]>,
) -> PathBuf {
    let components = children
        .as_ref()
        .iter()
        .map(|child| Component::Normal(child.as_ref()));
    parent.as_ref().components().chain(components).collect()
}

fn main() -> anyhow::Result<()> {
    let args = wild::args_os();
    let cli = Cli::parse_from(args);

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
        .or_else(|| Some(extend_path(dirs::data_dir()?, ["jim"]))) // .local/share, %APPDATA%, etc.
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
        Command::List => cmd::list(base),
        Command::Get => cmd::get(base),
        Command::Add { paths } => cmd::add(base, &paths),
        Command::Set { instance } => cmd::set(base, &instance),
    }
}
