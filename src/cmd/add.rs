use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
fn get_instance_name(_path: impl AsRef<Path>) -> io::Result<OsString> {
    todo!()
}

pub fn add(base: impl AsRef<Path> + Sync, paths: &[PathBuf]) -> anyhow::Result<()> {
    use rayon::prelude::*;

    let names = paths.par_iter().map(get_instance_name);
    let paths = paths.par_iter().map(dunce::canonicalize);

    // remove duplicate inputs
    let instances: HashMap<_, _> = names
        .zip(paths)
        .map(|(name, path)| Ok((name?, path?)))
        .collect::<io::Result<_>>()?;

    let install_instance = |(name, path): (OsString, PathBuf)| -> anyhow::Result<()> {
        let timer = Instant::now();

        let (name_str, path_str) = (name.to_string_lossy(), path.to_string_lossy());
        anyhow::ensure!(path.try_exists()?, "Input path does not exist");

        // this is where the instance would go
        let dest = crate::path_to_subdir(&base, &name);
        anyhow::ensure!(!dest.try_exists()?, "Instance {name_str} already installed");

        log::debug!("installing instance {name_str}");
        let copy_timer = Instant::now();
        dircpy::CopyBuilder::new(&path, &dest)
            .run()
            .with_context(|| {
                format!(
                    "Failed to copy instance from {path_str} to {}",
                    dest.to_string_lossy()
                )
            })?;
        let copy_time = copy_timer.elapsed().as_millis();
        log::trace!("installed {name_str} in {copy_time}ms");

        if atty::is(atty::Stream::Stdout) {
            let time = timer.elapsed().as_millis();
            println!("successfully installed {name_str} ({time}ms)");
        }

        Ok(())
    };

    instances.into_par_iter().map(install_instance).collect()
}
