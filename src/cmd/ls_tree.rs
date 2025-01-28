use super::{GitObject, Result};

pub fn run(hash: String, name_only: bool) -> Result<()> {
    let obj = GitObject::open_from_hash(&hash)?;
    obj.print_trees(name_only);
    Ok(())
}
