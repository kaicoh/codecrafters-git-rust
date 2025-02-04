mod args;
mod cmd;
mod error;
mod git_object;
mod git_protocol;
mod hash;
mod tree;

const GIT_DIR: &str = ".git";
const GIT_OBJ_DIR: &str = ".git/objects";
const GIT_REF_DIR: &str = ".git/refs";

use args::Args;
pub use cmd::Command;
pub use error::Error;
use git_object::GitObject;
use hash::{Sha1Hash, SHA1_HASH_SIZE};
pub type Result<T> = std::result::Result<T, Error>;
