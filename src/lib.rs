mod args;
mod cmd;
mod error;
mod git_object;

const GIT_DIR: &str = ".git";
const GIT_OBJ_DIR: &str = ".git/objects";
const GIT_REF_DIR: &str = ".git/refs";

use args::Args;
pub use cmd::Command;
pub use error::Error;
use git_object::GitObject;
pub type Result<T> = std::result::Result<T, Error>;
