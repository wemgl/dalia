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

const USAGE: &str = r#"Usage: dalia <command> [arguments]

Commands:
    aliases: Generates all shell aliases for each configured directory at DALIA_CONFIG_PATH
    help: Prints this usage message
    
Examples:
$ dalia aliases

Environment:
DALIA_CONFIG_PATH
    The location where dalia looks for alias configurations. This is set to $HOME/dalia by default.
    Put the alias configurations in a file named `config` here. 
    
Use "dalia help <command> for more information about that command.
"#;

const ALIASES_USAGE: &str = r#"Usage: dalia aliases

Description:
Aliases generates shell aliases for each directory listed in DALIA_CONFIG_PATH/config.
The aliases are only for changing directories to the specified locations. No other types
of aliases are supported.

Each alias outputted by this command is of the form `alias path="cd /some/path"`.

The configuration file uses its own format to generate aliases. The simplest way to generate
an alias to a directory is to provide its absolute path on disk. The generated alias will use
the lowercase name of directory at the end of the absolute path as the name of the alias. The
alias name can be customized as well, by prepending the absolute path with a custom name surrounded
by square brackets (i.e. `[` and `]`). The casing of the custom name doesn't change, so if it's
provided in titlecase, snakecase, or any other case, the alias will be created with that case in
tact.

This command also expands a single directory into multiple aliases when the configured line starts with
an asterisk surrounded by square brackets (i.e. `[*]`), which tells the parser to traverse the immediate
children of the given directory and create lowercase named aliases for only the items that are directories.
All children that are files are ignored. 

Examples:
    Simple path
    /some/path => alias path='cd /some/path'
    
    Custom name
    [my-path]/some/path => alias my-path='cd /some/path'
    [MyPath]/some/path => alias MyPath='cd /some/path'
    
    Directory Expansion
    [*]/some/path =>
        alias one='cd /some/path/one'
        alias two='cd /some/path/two'
        alias three='cd /some/path/three'
        
    when /some/path has contents /one, /two, file.txt, and /three.
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
    if args.is_empty() || args.len() > 3 {
        eprintln!("dalia: wrong number of arguments provided.");
        print_usage();
        exit(-1)
    }

    match args.get(1) {
        Some(cmd) => run_command(cmd, &args[1..args.len()]),
        None => {
            print_usage();
            exit(-1)
        }
    }
}

fn run_command(cmd: &str, args: &[String]) {
    if "aliases" == cmd {
        exit(match run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("dalia: {}", e);
                print_usage();
                1
            }
        })
    } else if "help" == cmd {
        if args.len() == 2 {
            match args[1].as_str() {
                "aliases" => print_alias_usage(),
                _ => {
                    eprintln!("dalia: unknown command argument {}\n", args[1]);
                    print_usage();
                }
            }
        } else {
            print_usage()
        }
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
            if contents.is_empty() {
                return Err(
                    "configuration file is empty; add a few paths to $DALIA_CONFIG_PATH/config and try again.".to_string(),
                );
            }
            Ok(Configuration::new(
                config_filepath.to_string(),
                Parser::new(&contents),
            ))
        }
        Err(_) => Err("missing configuration file; create configs for dalia to load at $DALIA_CONFIG_PATH/config".to_string()),
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

fn print_alias_usage() {
    println!("{}", ALIASES_USAGE)
}
