use std::env;
use std::process;

use hack_assembler::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(err) = hack_assembler::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1);
    }
}
