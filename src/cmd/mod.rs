mod cat_file;
mod hash_object;
mod init;
mod ls_tree;

use super::{Args, Error, GitObject, Result, GIT_DIR, GIT_OBJ_DIR, GIT_REF_DIR};

#[derive(Debug)]
pub enum Command {
    Init,
    CatFile { hash: String },
    HashObject { path: String },
    LsTree { hash: String, name_only: bool },
    Unknown,
}

impl Command {
    pub fn new(args: &[String]) -> Result<Self> {
        let cmd = match args.first().map(|v| v.as_str()) {
            Some("init") => Self::Init,
            Some("cat-file") => {
                let args = Args::builder().arg("-p").build(&args[1..]);
                let hash = args
                    .value("-p")
                    .ok_or(Error::from(anyhow::anyhow!("argument \"p\" is required")))?;
                Self::CatFile { hash }
            }
            Some("hash-object") => {
                let args = Args::builder().arg("-w").build(&args[1..]);
                let path = args
                    .value("-w")
                    .ok_or(Error::from(anyhow::anyhow!("argument \"w\" is required")))?;
                Self::HashObject { path }
            }
            Some("ls-tree") => {
                let args = Args::builder()
                    .flag("--name-only")
                    .position(0, "hash")
                    .build(&args[1..]);
                let name_only = args.flag("--name-only");
                let hash = args.value("hash").ok_or(Error::from(anyhow::anyhow!(
                    "position argument tree_sha is required"
                )))?;

                Self::LsTree { name_only, hash }
            }
            _ => Self::Unknown,
        };
        Ok(cmd)
    }

    pub fn run(self) -> Result<()> {
        match self {
            Self::Init => init::run(),
            Self::CatFile { hash } => cat_file::run(hash),
            Self::HashObject { path } => hash_object::run(path),
            Self::LsTree { hash, name_only } => ls_tree::run(hash, name_only),
            Self::Unknown => Err(anyhow::anyhow!("Unknown command").into()),
        }
    }
}
