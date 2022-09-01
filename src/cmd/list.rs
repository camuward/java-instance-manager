use std::fs;
use std::path::Path;

pub fn list(base: impl AsRef<Path>) -> anyhow::Result<()> {
    for instance in fs::read_dir(&base)?.flatten() {
        let meta = instance.metadata()?;
        if meta.is_dir() && !meta.is_symlink() {
            println!("{}", instance.file_name().to_string_lossy());
        }
    }

    Ok(())
}
