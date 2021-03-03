use std::collections::HashMap;
use std::{env, fs};

use crate::parser::Parser;

const DALIA_CONFIG_ENV_VAR: &str = "DALIA_CONFIG_PATH";
const CONFIG_FILE: &str = "config";
const DEFAULT_DALIA_CONFIG_PATH: &str = "~/.dalia";
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const USAGE: &str = r#"Usage: dalia <command> [arguments]

Commands:
    aliases: Generates all shell aliases for each configured directory at DALIA_CONFIG_PATH
    version: The current build version
    help: Prints this usage message
    
Examples:
    $ dalia aliases

Environment:
DALIA_CONFIG_PATH
    The location where dalia looks for alias configurations. This is set to $HOME/dalia by default.
    Put the alias configurations in a file named `config` here. 
    
Use "dalia help <command> for more information about that command."#;

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
        
    when /some/path has contents /one, /two, file.txt, and /three."#;

const VERSION_USAGE: &str = r#"Usage: dalia version

Description:
    Version prints the current semantic version of the dalia executable."#;

#[derive(Debug)]
struct Configuration<'a> {
    path: String,
    parser: Parser<'a>,
}

impl<'a> Configuration<'a> {
    fn new() -> Result<Configuration<'a>, &'static str> {
        let path = env::var(DALIA_CONFIG_ENV_VAR)
            .unwrap_or_else(|_| shellexpand::tilde(DEFAULT_DALIA_CONFIG_PATH).to_string());

        let path = format!("{}{}{}", path, std::path::MAIN_SEPARATOR, CONFIG_FILE);
        let contents = fs::read_to_string(&path).unwrap_or_default();
        if contents.is_empty() {
            return Err("configuration file is empty; add a few paths to $DALIA_CONFIG_PATH/config and try again.");
        }

        let parser = Parser::new(&contents);

        Ok(Configuration { path, parser })
    }

    fn aliases(&self) -> HashMap<String, String> {
        self.parser.aliases()
    }

    fn process_input(&mut self) -> Result<(), String> {
        self.parser.process_input()
    }
}

pub enum Command {
    Aliases,
    Version,
    Help,
}

impl Command {
    pub fn run(args: Vec<String>) -> Result<(), String> {
        if args.is_empty() || args.len() > 3 {
            return Err("wrong number of arguments provided.".to_string());
        } else if args.len() == 1 {
            print_usage();
            return Ok(());
        }

        let cmd = args.get(1).unwrap();
        match Command::from_str(cmd) {
            Some(Command::Aliases) => generate_aliases(),
            Some(Command::Version) => {
                print_version();
                Ok(())
            }
            Some(Command::Help) => {
                if args.len() == 3 {
                    return print_help(args[2].as_str());
                } else {
                    print_usage();
                }
                Ok(())
            }
            None => {
                return Err(format!("unknown command: {}", cmd));
            }
        }
    }

    fn from_str(value: &str) -> Option<Command> {
        match value {
            "aliases" => Some(Command::Aliases),
            "version" => Some(Command::Version),
            "help" => Some(Command::Help),
            _ => None,
        }
    }
}

fn print_help(value: &str) -> Result<(), String> {
    match Command::from_str(value) {
        Some(Command::Aliases) => print_alias_usage(),
        Some(Command::Version) => print_version_usage(),
        Some(Command::Help) => print_usage(),
        None => {
            return Err(format!("unknown command: {}", value));
        }
    }
    Ok(())
}

fn generate_aliases() -> Result<(), String> {
    let mut config = Configuration::new()?;
    config.process_input()?;

    let aliases: Vec<String> = config
        .aliases()
        .iter()
        .map(|(alias, path)| format!("alias {}='cd {}'\n", alias, path))
        .collect();

    aliases.iter().for_each(|alias| print!("{}", alias));

    Ok(())
}

fn print_usage() {
    println!("{}", USAGE)
}

fn print_alias_usage() {
    println!("{}", ALIASES_USAGE)
}

fn print_version_usage() {
    println!("{}", VERSION_USAGE)
}

fn print_version() {
    if let Some(v) = VERSION {
        println!("dalia version {}", v)
    }
}
