use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Context;

/// Determine the instance name from an input path.
///
/// If `path` points to an archive, the extension is stripped.
///
/// If `path` is a directory containing a `bin` subfolder, the instance is
/// named the same as `path`.
///
/// # Usage
///
/// ```no_run
/// let file = "/home/user/Downloads/openjdk-19_linux-x64_bin.tar.gz";
/// assert_eq!("openjdk-19_linux-x64_bin", get_instance_name(file));
///
/// let folder = "/home/user/Desktop/graalvm-ee-java17-22.2.0/";
/// assert_eq!("graalvm-ee-java17-22.2.0", get_instance_name(folder));
/// ```
fn get_instance_name(path: impl AsRef<Path>) -> anyhow::Result<OsString> {}

pub fn add(base: impl AsRef<Path> + Sync, paths: &[PathBuf]) -> anyhow::Result<()> {
    use rayon::prelude::*;

    let instance_names: HashSet<OsString> = paths.iter().map(get_instance_name).collect()?;

    // remove duplicate inputs
    let real_paths: HashSet<_> = paths.par_iter().map(dunce::canonicalize).collect()?;

    real_paths
        .zip(file_names)
        .map(|(path, file_name)| {
            use std::time::Instant;

            let start = Instant::now();

            let (input_path, file_name) =
                (path?, file_name.context("Failed to get input filename")?);
            let name = file_name.to_string_lossy();

            log::debug!(
                target: &name,
                "searching for input directory at {}...",
                input_path.to_string_lossy()
            );
            anyhow::ensure!(input_path.try_exists()?, "Input path does not exist");

            let instance: PathBuf = crate::path_to_subdir(&base, file_name);
            anyhow::ensure!(!instance.try_exists()?, "Instance is already installed");

            log::debug!("installing instance {name}");
            let copy_start = Instant::now();
            dircpy::CopyBuilder::new(&input_path, &instance)
                .run()
                .with_context(|| {
                    format!(
                        "Failed to copy instance from {} to {}",
                        input_path.to_string_lossy(),
                        instance.to_string_lossy()
                    )
                })?;
            log::trace!(
                "installed {} in {}ms",
                instance.to_string_lossy(),
                copy_start.elapsed().as_millis()
            );

            if atty::is(atty::Stream::Stdout) {
                println!(
                    "successfully installed {name} ({}ms)",
                    start.elapsed().as_millis(),
                );
            }

            Ok(())
        })
        .collect()
}
