use std::path::{Path, PathBuf};

use anyhow::Context;

pub fn add(base: impl AsRef<Path> + Sync, paths: &[PathBuf]) -> anyhow::Result<()> {
    use rayon::prelude::*;

    // remove duplicate paths (even through symlinks)
    let real_paths = paths.par_iter().map(dunce::canonicalize);
    let file_names = paths.par_iter().map(AsRef::as_ref).map(Path::file_name);

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

            let instance: PathBuf = crate::extend_path(&base, [file_name]);
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
