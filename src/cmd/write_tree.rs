use super::{GitObject, Result};

pub fn run() -> Result<()> {
    let obj = GitObject::new_tree(".")?;
    print!("{}", obj.hash().hex());
    obj.write(".")
}
