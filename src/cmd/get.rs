use std::path::Path;

use anyhow::Context;

pub fn get(base: impl AsRef<Path>) -> anyhow::Result<()> {
    if let Some(path) = crate::cmd::path_to_current_instance(&base)? {
        let name = path
            .file_name()
            .context("Failed to get instance filename")?
            .to_string_lossy();

        println!("{name}");
    } else {
        log::info!("current instance symlink does not exist");
        std::process::exit(1);
    }

    Ok(())
}
