use super::{GitObject, Result};

pub fn run(hash: String, name_only: bool) -> Result<()> {
    let obj = GitObject::open_from_hash(".", &hash)?;
    for tree in obj.print_trees(name_only) {
        println!("{tree}");
    }
    Ok(())
}
