use super::{Result, GIT_DIR, GIT_OBJ_DIR, GIT_REF_DIR};
use std::fs;

pub(crate) fn run() -> Result<()> {
    fs::create_dir(GIT_DIR)?;
    fs::create_dir(GIT_OBJ_DIR)?;
    fs::create_dir(GIT_REF_DIR)?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}
