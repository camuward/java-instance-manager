use std::ffi::OsString;
use std::path::Path;

use anyhow::Context;

pub fn set(base: impl AsRef<Path>, name: &OsString) -> anyhow::Result<()> {
    // check instance exists
    log::debug!("searching for {}", name.to_string_lossy());
    let instance = crate::path_to_subdir(&base, &name);
    anyhow::ensure!(
        instance.try_exists()?,
        "Instance {} does not exist",
        instance.to_string_lossy()
    );
    log::debug!("found {}", instance.to_string_lossy());

    log::info!("setting current instance to {}", name.to_string_lossy());

    // remove existing symlink at $JIM_DIR/current
    let link = crate::path_to_subdir(base, "current");
    if link.try_exists()? {
        log::debug!("symlink exists, removing...");
        symlink::remove_symlink_dir(&link).with_context(|| {
            format!(
                "Failed to remove `current` symlink ({})",
                link.to_string_lossy()
            )
        })?;
    }

    // create symlink to instance at $JIM_DIR/current
    log::debug!("creating symlink...");
    symlink::symlink_dir(instance, &link).with_context(|| {
        format!(
            "Failed to create `current` symlink ({})",
            link.to_string_lossy()
        )
    })
}
