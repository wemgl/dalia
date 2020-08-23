extern crate shellexpand;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::{exit, Command};

use parser::Parser;

mod parser;

const DALIA_CONFIG_ENV_VAR: &str = "DALIA_CONFIG_PATH";
const CONFIG_FILE: &str = "config";
const DEFAULT_DALIA_CONFIG_PATH: &str = "~/.dalia";
const ALIAS_PROGRAM: &str = "alias";

#[derive(Debug)]
struct Configuration<'a> {
    path: String,
    parser: parser::Parser<'a>,
}

impl<'a> Configuration<'a> {
    fn new(path: String, parser: parser::Parser<'a>) -> Self {
        Self { path, parser }
    }

    fn aliases(&self) -> HashMap<String, String> {
        self.parser.aliases()
    }

    fn process_input(&mut self) -> Result<(), String> {
        self.parser.process_input()
    }
}

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("dalia: {}", e);
            1
        }
    })
}

fn load_configuration<'a>() -> Result<Configuration<'a>, String> {
    let config_path = match env::var(DALIA_CONFIG_ENV_VAR) {
        Ok(val) => val,
        Err(_) => shellexpand::tilde(DEFAULT_DALIA_CONFIG_PATH).to_string(),
    };

    let config_filepath =
        &*(config_path.to_owned() + std::path::MAIN_SEPARATOR.to_string().as_str() + CONFIG_FILE);

    match File::open(config_filepath) {
        Ok(mut config_file) => {
            let mut contents = String::new();
            if let Err(e) = config_file.read_to_string(&mut contents) {
                return Err(e.to_string());
            }
            Ok(Configuration::new(
                config_filepath.to_string(),
                Parser::new(&contents),
            ))
        }
        Err(e) => Err(format!("missing configuration file: {}", e)),
    }
}

fn run() -> Result<(), String> {
    let mut config = load_configuration()?;
    config.process_input()?;
    let mut alias_cmd = Command::new(ALIAS_PROGRAM);
    for (alias, path) in config.aliases() {
        alias_cmd.arg(format!(r#"{}=cd "{}""#, alias, path));
    }
    let status = alias_cmd
        .status()
        .expect("alias process failed to create aliases");
    if !status.success() {
        return Err(format!("couldn't create aliases: {}", status));
    }
    Ok(())
}
