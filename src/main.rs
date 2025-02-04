use codecrafters_git::{Command, Result};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(err) = run(&args[1..]).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run(args: &[String]) -> Result<()> {
    let cmd = Command::new(args)?;
    cmd.run().await
}
