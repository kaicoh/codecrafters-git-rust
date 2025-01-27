mod cat_file;
mod init;

use super::{Args, Error, GitObject, Result, GIT_DIR, GIT_OBJ_DIR, GIT_REF_DIR};

#[derive(Debug)]
pub enum Command {
    Init,
    CatFile { hash: String },
    Unknown,
}

impl Command {
    pub fn new(args: &[String]) -> Result<Self> {
        let cmd = match args.first().map(|v| v.as_str()) {
            Some("init") => Self::Init,
            Some("cat-file") => {
                let args = Args::new(&args[1..]);
                let hash = args
                    .value("p")
                    .ok_or(Error::from(anyhow::anyhow!("argument \"p\" is required")))?;
                Self::CatFile { hash }
            }
            _ => Self::Unknown,
        };
        Ok(cmd)
    }

    pub fn run(self) -> Result<()> {
        match self {
            Self::Init => init::run(),
            Self::CatFile { hash } => cat_file::run(hash),
            Self::Unknown => Err(anyhow::anyhow!("Unknown command").into()),
        }
    }
}
