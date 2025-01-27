use super::{GitObject, Result};
use std::fs::File;

pub(crate) fn run(path: String) -> Result<()> {
    let f = File::open(path)?;
    let obj = GitObject::new_blob(f)?;
    print!("{}", obj.hash());
    obj.write()
}
