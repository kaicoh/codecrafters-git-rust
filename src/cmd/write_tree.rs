use super::{GitObject, Result};

pub fn run() -> Result<()> {
    let obj = GitObject::new_tree(".")?;
    print!("{}", hex::encode(obj.hash()));
    obj.write()
}
