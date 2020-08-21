mod parser;

use parser::Parser;

const DEFAULT_DALIA_CONFIG_PATH: &str = "~/.dalia";

#[derive(Debug)]
struct Configuration<'a> {
    path: String,
    parser: parser::Parser<'a>,
}

impl<'a> Configuration<'a> {
    fn new(path: String, parser: parser::Parser<'a>) -> Self {
        Self { path, parser }
    }
}

fn main() {
    let config_file = "/test/path";
    let mut config = Configuration::new(DEFAULT_DALIA_CONFIG_PATH.into(), Parser::new(config_file));
    std::process::exit(match run(&mut config) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("dalia: run failed unexpectedly: {}", e);
            1
        }
    })
}

fn run(config: &mut Configuration) -> Result<(), String> {
    match config.parser.file() {
        Ok(_) => {
            println!("{:?}", config);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
