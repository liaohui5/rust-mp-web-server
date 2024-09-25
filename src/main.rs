use rust_http_server::*;
use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err|{
        eprintln!("Failed to parse arguments: {}", err);
        process::exit(1);
    });

    http_server::listen(config).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(0);
    });
}

