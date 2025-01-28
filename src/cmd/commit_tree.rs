use super::{GitObject, Result};

pub fn run(tree: String, comment: String, parent: Option<String>) -> Result<()> {
    let obj = GitObject::new_commit(tree, comment, parent)?;
    print!("{}", hex::encode(obj.hash()));
    obj.write()
}
