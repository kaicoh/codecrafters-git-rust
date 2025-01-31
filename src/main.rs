use codecrafters_git::Command;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(err) = Command::new(&args[1..]).and_then(Command::run) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
