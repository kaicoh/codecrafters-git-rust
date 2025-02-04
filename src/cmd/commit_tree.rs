use super::{GitObject, Result};

pub fn run(tree: String, comment: String, parent: Option<String>) -> Result<()> {
    let obj = GitObject::new_commit(tree, comment, parent.into_iter().collect())?;
    print!("{}", obj.hash().hex());
    obj.write(".")
}
