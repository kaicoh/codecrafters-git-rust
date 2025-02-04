use super::{GitObject, Result};

pub(crate) fn run(hash: String) -> Result<()> {
    let obj = GitObject::open_from_hash(".", &hash)?;
    print!("{obj}");
    Ok(())
}
