extern crate shellexpand;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use parser::Parser;

mod parser;

const DALIA_CONFIG_ENV_VAR: &str = "DALIA_CONFIG_PATH";
const CONFIG_FILE: &str = "config";
const DEFAULT_DALIA_CONFIG_PATH: &str = "~/.dalia";

const USAGE: &str = r#"Usage: dalia <command>
    aliases: Generates all shell aliases for each configured directory at DALIA_CONFIG_PATH
    help: Prints this usage message
    
Examples:
$ dalia aliases
"#;

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
    let args: Vec<String> = env::args().collect();
    if args.is_empty() || args.len() > 2 {
        eprintln!("dalia: wrong number of arguments provided.");
        print_usage();
        exit(-1)
    }

    match args.get(1) {
        Some(cmd) => run_command(cmd),
        None => {
            print_usage();
            exit(-1)
        }
    }
}

fn run_command(cmd: &String) {
    if &String::from("aliases") == cmd {
        exit(match run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("dalia: {}", e);
                print_usage();
                1
            }
        })
    } else if &String::from("help") == cmd {
        print_usage()
    } else {
        eprintln!("dalia: unknown command {}", cmd);
        print_usage();
    }
}

fn load_configuration<'a>() -> Result<Configuration<'a>, String> {
    let config_path = env::var(DALIA_CONFIG_ENV_VAR)
        .unwrap_or_else(|_| shellexpand::tilde(DEFAULT_DALIA_CONFIG_PATH).to_string());

    let config_filepath =
        &*(config_path + std::path::MAIN_SEPARATOR.to_string().as_str() + CONFIG_FILE);

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

    let mut aliases = Vec::new();
    for (alias, path) in config.aliases() {
        aliases.push(format!("alias {}='cd {}'\n", alias, path));
    }

    for alias in aliases {
        print!("{}", alias)
    }

    Ok(())
}

fn print_usage() {
    println!("{}", USAGE)
}
