use super::{Result, GIT_DIR, GIT_OBJ_DIR, GIT_REF_DIR};
use std::{fs, path::Path};

pub(crate) fn run<P: AsRef<Path>>(root: P) -> Result<()> {
    let path = root.as_ref();
    fs::create_dir(path.join(GIT_DIR))?;
    fs::create_dir(path.join(GIT_OBJ_DIR))?;
    fs::create_dir(path.join(GIT_REF_DIR))?;
    fs::write(path.join(GIT_DIR).join("HEAD"), "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}
