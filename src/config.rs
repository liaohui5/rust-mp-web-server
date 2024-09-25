use std::num::ParseIntError;

pub struct Config {
    pub port: u16,
    pub path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, ParseIntError> {
        let mut config = Config {
            port: 8080,
            path: String::from("./public"),
        };

        if args.len() >= 2 {
            for item in args.iter() {
                // --port
                if item == "--port" {
                    config.port = item.clone().parse()?;
                }

                // --dir or --path or --public-dir or --public-path
                if item == "--dir" || item == "--path" || item == "--public-dir" || item == "--public-path" {
                    config.path = item.to_string();
                }
            }
        }

        Ok(config)
    }
}
